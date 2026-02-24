# Go — Orsta-Client Usage

Install dependency:
```bash
go get github.com/gorilla/websocket
```

---

## 1. Sign Up

```go
package main

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
)

func signup() (string, error) {
	payload := map[string]interface{}{
		"username": "alice",
		"email":    "alice@example.com",
		"password": "supersecret",
		"passkey":  nil,
	}
	body, _ := json.Marshal(payload)

	resp, err := http.Post(
		"http://localhost:3000/auth/signup",
		"application/json",
		bytes.NewReader(body),
	)
	if err != nil {
		return "", err
	}
	defer resp.Body.Close()

	var result map[string]interface{}
	data, _ := io.ReadAll(resp.Body)
	json.Unmarshal(data, &result)

	fmt.Printf("User created: %v\n", result)
	return result["token"].(string), nil
}
```

---

## 2. Log In

```go
func login() (string, error) {
	payload := map[string]string{
		"email":    "alice@example.com",
		"password": "supersecret",
	}
	body, _ := json.Marshal(payload)

	resp, err := http.Post(
		"http://localhost:3000/auth/login",
		"application/json",
		bytes.NewReader(body),
	)
	if err != nil {
		return "", err
	}
	defer resp.Body.Close()

	var result map[string]interface{}
	data, _ := io.ReadAll(resp.Body)
	json.Unmarshal(data, &result)

	return result["token"].(string), nil
}
```

---

## 3. WebSocket Connection

```go
package main

import (
	"encoding/json"
	"fmt"
	"log"
	"net/http"

	"github.com/gorilla/websocket"
)

type WsMessage struct {
	Action  string      `json:"action"`
	Payload interface{} `json:"payload,omitempty"`
}

func connectWS(token string) {
	headers := http.Header{}
	headers.Set("Authorization", "Bearer "+token)

	conn, _, err := websocket.DefaultDialer.Dial(
		"ws://localhost:3000/ws",
		headers,
	)
	if err != nil {
		log.Fatal("WebSocket dial error:", err)
	}
	defer conn.Close()

	// Send ping
	msg := WsMessage{Action: "ping"}
	data, _ := json.Marshal(msg)
	conn.WriteMessage(websocket.TextMessage, data)

	// Read response
	for {
		_, raw, err := conn.ReadMessage()
		if err != nil {
			log.Println("Read error:", err)
			break
		}
		var resp map[string]interface{}
		json.Unmarshal(raw, &resp)
		fmt.Printf("Received: %v\n", resp)
	}
}

func main() {
	token, err := login()
	if err != nil {
		log.Fatal(err)
	}
	connectWS(token)
}
```

---

## 4. Log Out

```go
func logout() {
	http.Post("http://localhost:3000/auth/logout", "application/json", nil)
}
```
