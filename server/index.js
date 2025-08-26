import express from "express";
import fs from "fs";
import cors from "cors";
import path from "path";

const app = express();
const PORT = process.env.PORT || 3000;

app.use(cors());
app.use(express.json());

const DATA_FILE = path.join(process.cwd(), "games.json");

// --- Get all games ---
app.get("/games", (req, res) => {
    fs.readFile(DATA_FILE, "utf-8", (err, data) => {
        if (err) return res.status(500).json({ error: "Failed to read file" });
        try {
            res.json(JSON.parse(data));
        } catch {
            res.json([]);
        }
    });
});

// --- Add a new game ---
app.post("/games", (req, res) => {
    const newGame = req.body;
    if (!newGame.title || !newGame.path) {
        return res.status(400).json({ error: "title and path are required" });
    }

    fs.readFile(DATA_FILE, "utf-8", (err, data) => {
        let games = [];
        if (!err && data) {
            try { games = JSON.parse(data); } catch { }
        }

        games.push({
            ...newGame,
            added: new Date().toISOString()
        });

        fs.writeFile(DATA_FILE, JSON.stringify(games, null, 2), (err) => {
            if (err) return res.status(500).json({ error: "Failed to write file" });
            res.json({ success: true, games });
        });
    });
});

app.listen(PORT, () => {
    console.log(`Server running on port ${PORT}`);
});