package llm

import (
	"os/exec"
	"regexp"
	"runtime"
	"strconv"
	"strings"
)

// SystemInfo holds detected hardware and model recommendation
type SystemInfo struct {
	GPUType   string `json:"gpu_type"`   // nvidia | amd_rocm | amd_no_rocm | intel_arc | cpu
	GPUName   string `json:"gpu_name"`
	VRAMGB    int    `json:"vram_gb"`
	RAMGB     int    `json:"ram_gb"`
	RecModel  string `json:"rec_model"`
	RecSize   string `json:"rec_size"`
	RecReason string `json:"rec_reason"`
}

// DetectSystem detects GPU hardware and recommends an appropriate model
func DetectSystem() *SystemInfo {
	info := &SystemInfo{GPUType: "cpu"}
	info.RAMGB = detectRAM()

	if detectNVIDIA(info) {
		recommendModel(info)
		return info
	}
	if detectAMD(info) {
		recommendModel(info)
		return info
	}
	if detectIntelArc(info) {
		recommendModel(info)
		return info
	}

	info.GPUName = "No discrete GPU detected"
	recommendModel(info)
	return info
}

func detectNVIDIA(info *SystemInfo) bool {
	out, err := exec.Command("nvidia-smi",
		"--query-gpu=name,memory.total",
		"--format=csv,noheader,nounits").Output()
	if err != nil {
		return false
	}

	line := strings.TrimSpace(string(out))
	if line == "" {
		return false
	}

	// Format: "NVIDIA GeForce RTX 3060, 12288"
	parts := strings.SplitN(line, ",", 2)
	info.GPUType = "nvidia"
	info.GPUName = strings.TrimSpace(parts[0])
	if len(parts) > 1 {
		if mb, err := strconv.Atoi(strings.TrimSpace(parts[1])); err == nil {
			info.VRAMGB = mb / 1024
		}
	}
	return true
}

func detectAMD(info *SystemInfo) bool {
	if runtime.GOOS != "linux" {
		return false
	}

	out, err := exec.Command("lspci").Output()
	if err != nil {
		return false
	}

	amdRe := regexp.MustCompile(`(?i)(vga|3d|display).*?(amd|radeon|advanced micro)`)
	lines := strings.Split(string(out), "\n")
	for _, line := range lines {
		if amdRe.MatchString(line) {
			// Extract GPU name after ": "
			if idx := strings.Index(line, ": "); idx >= 0 {
				info.GPUName = strings.TrimSpace(line[idx+2:])
			} else {
				info.GPUName = "AMD/Radeon GPU"
			}

			// Check for ROCm
			if hasROCm() {
				info.GPUType = "amd_rocm"
				info.VRAMGB = detectROCmVRAM()
				if info.VRAMGB == 0 {
					info.VRAMGB = estimateAMDVRAM(info.GPUName)
				}
			} else {
				info.GPUType = "amd_no_rocm"
				info.VRAMGB = estimateAMDVRAM(info.GPUName)
			}
			return true
		}
	}
	return false
}

func hasROCm() bool {
	_, err := exec.LookPath("rocm-smi")
	return err == nil
}

func detectROCmVRAM() int {
	out, err := exec.Command("rocm-smi", "--showmeminfo", "vram").Output()
	if err != nil {
		return 0
	}

	// Look for total line with a number
	totalRe := regexp.MustCompile(`(?i)total.*?(\d+)`)
	m := totalRe.FindStringSubmatch(string(out))
	if len(m) < 2 {
		return 0
	}

	val, err := strconv.Atoi(m[1])
	if err != nil {
		return 0
	}

	// rocm-smi may report bytes or MB
	if val > 100000 {
		return val / (1024 * 1024) // bytes to GB
	}
	return val / 1024 // MB to GB
}

func estimateAMDVRAM(name string) int {
	lower := strings.ToLower(name)
	estimates := []struct {
		pattern string
		vramGB  int
	}{
		{"7900 xtx", 24}, {"7900 xt", 20}, {"7900 gre", 16},
		{"7800 xt", 16}, {"7700 xt", 12}, {"7600", 8},
		{"6950 xt", 16}, {"6900 xt", 16}, {"6800 xt", 16}, {"6800", 16},
		{"6700 xt", 12}, {"6700", 10}, {"6600 xt", 8}, {"6600", 8},
		{"6500 xt", 4}, {"580", 8}, {"570", 8}, {"vega", 8},
	}
	for _, e := range estimates {
		if strings.Contains(lower, e.pattern) {
			return e.vramGB
		}
	}
	return 0
}

func detectIntelArc(info *SystemInfo) bool {
	if runtime.GOOS != "linux" {
		return false
	}

	out, err := exec.Command("lspci").Output()
	if err != nil {
		return false
	}

	arcRe := regexp.MustCompile(`(?i)(vga|3d|display).*?intel.*(arc|dg[12])`)
	lines := strings.Split(string(out), "\n")
	for _, line := range lines {
		if arcRe.MatchString(line) {
			info.GPUType = "intel_arc"
			if idx := strings.Index(line, ": "); idx >= 0 {
				info.GPUName = strings.TrimSpace(line[idx+2:])
			} else {
				info.GPUName = "Intel Arc GPU"
			}
			return true
		}
	}
	return false
}

func detectRAM() int {
	if runtime.GOOS != "linux" {
		return 0
	}

	out, err := exec.Command("free", "--giga").Output()
	if err != nil {
		return 0
	}

	// "Mem:    31 ..."
	memRe := regexp.MustCompile(`Mem:\s+(\d+)`)
	m := memRe.FindStringSubmatch(string(out))
	if len(m) < 2 {
		return 0
	}

	gb, _ := strconv.Atoi(m[1])
	return gb
}

func recommendModel(info *SystemInfo) {
	vram := info.VRAMGB

	switch info.GPUType {
	case "nvidia":
		switch {
		case vram >= 24:
			info.RecModel = "llama3.1:70b-instruct-q4_0"
			info.RecSize = "~40 GB"
			info.RecReason = "Best quality for " + strconv.Itoa(vram) + "GB VRAM"
		case vram >= 12:
			info.RecModel = "llama3.1:8b"
			info.RecSize = "4.7 GB"
			info.RecReason = "Sweet spot for " + strconv.Itoa(vram) + "GB VRAM"
		case vram >= 8:
			info.RecModel = "llama3.1:8b-instruct-q4_0"
			info.RecSize = "~4 GB"
			info.RecReason = "Quantized fit for " + strconv.Itoa(vram) + "GB VRAM"
		case vram >= 6:
			info.RecModel = "llama3.2:3b"
			info.RecSize = "2 GB"
			info.RecReason = "Compact model for " + strconv.Itoa(vram) + "GB VRAM"
		default:
			info.RecModel = "llama3.2:1b"
			info.RecSize = "1 GB"
			info.RecReason = "Minimal model for limited VRAM"
		}

	case "amd_rocm":
		if vram >= 16 {
			info.RecModel = "llama3.1:8b"
			info.RecSize = "4.7 GB"
			info.RecReason = "ROCm-accelerated, great for " + strconv.Itoa(vram) + "GB VRAM"
		} else {
			info.RecModel = "llama3.2:3b"
			info.RecSize = "2 GB"
			info.RecReason = "Conservative fit for Radeon with ROCm"
		}

	case "amd_no_rocm":
		// Without ROCm, use CPU-based sizing
		info.recommendCPU()

	case "intel_arc":
		info.RecModel = "llama3.2:3b"
		info.RecSize = "2 GB"
		info.RecReason = "Intel Arc support is experimental"

	case "cpu":
		info.recommendCPU()
	}
}

func (info *SystemInfo) recommendCPU() {
	ram := info.RAMGB
	switch {
	case ram >= 32:
		info.RecModel = "llama3.1:8b"
		info.RecSize = "4.7 GB"
		info.RecReason = "CPU mode \u2014 " + strconv.Itoa(ram) + "GB RAM handles 8b"
	case ram >= 16:
		info.RecModel = "llama3.2:3b"
		info.RecSize = "2 GB"
		info.RecReason = "CPU mode \u2014 comfortable for " + strconv.Itoa(ram) + "GB RAM"
	default:
		info.RecModel = "llama3.2:1b"
		info.RecSize = "1 GB"
		info.RecReason = "CPU mode \u2014 minimal model for " + strconv.Itoa(ram) + "GB RAM"
	}
}
