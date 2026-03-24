package main

import (
	"context"
	"encoding/json"
	"fmt"
	"log"
	"net/http"
	"reflect"
)

// HTTPBridge exposes App methods via HTTP for browser-based testing
type HTTPBridge struct {
	app    *App
	server *http.Server
}

// NewHTTPBridge creates a new HTTP bridge for the given app on the specified port
func NewHTTPBridge(app *App, port int) *HTTPBridge {
	bridge := &HTTPBridge{app: app}
	mux := http.NewServeMux()
	bridge.registerAppMethods(mux)

	bridge.server = &http.Server{
		Addr:    fmt.Sprintf(":%d", port),
		Handler: corsMiddleware(mux),
	}
	return bridge
}

// Start begins listening for HTTP requests in a goroutine
func (b *HTTPBridge) Start() {
	go func() {
		log.Printf("HTTP bridge listening on %s", b.server.Addr)
		if err := b.server.ListenAndServe(); err != http.ErrServerClosed {
			log.Printf("HTTP bridge error: %v", err)
		}
	}()
}

// Stop gracefully shuts down the HTTP server
func (b *HTTPBridge) Stop(ctx context.Context) error {
	return b.server.Shutdown(ctx)
}

// corsMiddleware adds CORS headers for browser access
func corsMiddleware(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Access-Control-Allow-Origin", "*")
		w.Header().Set("Access-Control-Allow-Methods", "POST, OPTIONS")
		w.Header().Set("Access-Control-Allow-Headers", "Content-Type")
		if r.Method == "OPTIONS" {
			w.WriteHeader(http.StatusOK)
			return
		}
		next.ServeHTTP(w, r)
	})
}

// registerAppMethods uses reflection to register all exported App methods as HTTP handlers
func (b *HTTPBridge) registerAppMethods(mux *http.ServeMux) {
	appType := reflect.TypeOf(b.app)
	for i := 0; i < appType.NumMethod(); i++ {
		method := appType.Method(i)
		// Skip unexported and lifecycle methods
		if !method.IsExported() || method.Name == "startup" || method.Name == "shutdown" {
			continue
		}
		handler := b.createMethodHandler(method)
		path := "/api/" + method.Name
		mux.HandleFunc(path, handler)
		log.Printf("Registered HTTP endpoint: POST %s", path)
	}
}

// createMethodHandler creates an HTTP handler for a single App method
func (b *HTTPBridge) createMethodHandler(method reflect.Method) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		if r.Method != "POST" {
			http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
			return
		}
		w.Header().Set("Content-Type", "application/json")

		// Parse arguments from JSON array in request body
		var args []json.RawMessage
		if r.ContentLength > 0 {
			if err := json.NewDecoder(r.Body).Decode(&args); err != nil {
				json.NewEncoder(w).Encode(map[string]string{"error": err.Error()})
				return
			}
		}

		methodType := method.Type
		numIn := methodType.NumIn() - 1 // -1 for receiver
		if len(args) != numIn {
			json.NewEncoder(w).Encode(map[string]string{
				"error": fmt.Sprintf("expected %d args, got %d", numIn, len(args)),
			})
			return
		}

		// Build call arguments: receiver + parameters
		callArgs := []reflect.Value{reflect.ValueOf(b.app)}
		for i, argJSON := range args {
			paramType := methodType.In(i + 1)
			paramVal := reflect.New(paramType)
			if err := json.Unmarshal(argJSON, paramVal.Interface()); err != nil {
				json.NewEncoder(w).Encode(map[string]string{"error": err.Error()})
				return
			}
			callArgs = append(callArgs, paramVal.Elem())
		}

		// Call the method
		results := method.Func.Call(callArgs)

		// Check for error return
		for _, result := range results {
			if result.Type().Implements(reflect.TypeOf((*error)(nil)).Elem()) && !result.IsNil() {
				json.NewEncoder(w).Encode(map[string]string{
					"error": result.Interface().(error).Error(),
				})
				return
			}
		}

		// Return the first non-error result, or null
		if len(results) > 0 && !results[0].Type().Implements(reflect.TypeOf((*error)(nil)).Elem()) {
			json.NewEncoder(w).Encode(map[string]interface{}{
				"result": results[0].Interface(),
			})
		} else {
			json.NewEncoder(w).Encode(map[string]interface{}{"result": nil})
		}
	}
}
