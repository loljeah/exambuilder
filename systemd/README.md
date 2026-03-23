# Systemd User Service

## Installation

```bash
# Copy service file
mkdir -p ~/.config/systemd/user
cp systemd/kgate-daemon.service ~/.config/systemd/user/

# Copy binary
mkdir -p ~/.local/bin
cp kgate-daemon ~/.local/bin/
cp kgatectl ~/.local/bin/

# Enable and start
systemctl --user daemon-reload
systemctl --user enable kgate-daemon
systemctl --user start kgate-daemon
```

## Commands

```bash
# Check status
systemctl --user status kgate-daemon

# View logs
journalctl --user -u kgate-daemon -f

# Restart after update
systemctl --user restart kgate-daemon
```

## NixOS Home Manager

Add to your Home Manager configuration:

```nix
systemd.user.services.kgate-daemon = {
  Unit = {
    Description = "Knowledge Gate Daemon";
    After = [ "graphical-session.target" ];
    PartOf = [ "graphical-session.target" ];
  };
  Service = {
    Type = "simple";
    ExecStart = "${pkgs.kgate}/bin/kgate-daemon";
    Restart = "on-failure";
    RestartSec = 5;
  };
  Install.WantedBy = [ "graphical-session.target" ];
};
```
