# Websocket

Create game_id

$ curl -X POST http://127.0.0.1:8080/create

{"game_id":"d1ea6136-4d8f-4827-a842-f8a3ccb2a2f9"}

Join game_id, returns ws url

$ curl -X POST http://127.0.0.1:8080/join -H "Content-Type: Application/Json" -d '{"game_id":"d1ea6136-4d8f-4827-a842-f8a3ccb2a2f9"}'


{"url":"ws://127.0.0.1:8080/ws/caf933ed-0851-4e66-b576-29b3c5e17cbf"}

Finally connect,

$ wscat --connect "ws://127.0.0.1:8080/ws/caf933ed-0851-4e66-b576-29b3c5e17cbf"

Connected

< {"from":"server","message":"alive"}
