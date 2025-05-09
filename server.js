const express = require("express");
const expressWs = require('express-ws');
const app = express();
let users = {};
const PORT = 3000;
const { app: wsApp } = expressWs(app);
app.use(express.static('static'));
// app.get("/", (req, res) => {
//     res.send("Hello from Express!");
// });

app.ws('/ws/:username', (ws, req) => {
    let username = req.params.username;
    users[username] = ws;
    console.log('WebSocket connected!');

    ws.on('message', (msg) => {
        let message = JSON.parse(msg);
        users[message.to].send(JSON.stringify({ from: username, message: message.message }));
        console.log('Message from client:', msg);
    });

});

app.listen(PORT, () => {
    console.log(`Server is running on http://localhost:${PORT}`);
});
