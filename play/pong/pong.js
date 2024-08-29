const canvas = document.getElementById('gameCanvas');
const ctx = canvas.getContext('2d');

let bxLoc = 250, byLoc = 250, bxVel = Math.random() * 40 - 20, byVel = 10;
let pxLoc = 250, pyLoc = 400;
let score = 0;

function gameLoop() {
    // Update ball position
    bxLoc += bxVel;
    byLoc += byVel;

    // Paddle collision
    if (bxLoc + 30 >= pxLoc && bxLoc <= pxLoc + 170 && byLoc + 20 >= pyLoc && byLoc <= pyLoc + 20) {
        byVel *= -1;
        byLoc -= 10;
        score += 1;
    }

    // Right Side
    if (bxLoc > 470) {
        bxVel *= -1;
    }

    // Left Side
    if (bxLoc < 25) {
        bxVel *= -1;
    }

    // Bottom Side
    if (byLoc > 470) {
        resetBall();
        score = 0;
    }

    // Top Side
    if (byLoc < 25) {
        byVel *= -1;
    }

    // Clear canvas
    ctx.clearRect(0, 0, canvas.width, canvas.height);

    // Draw background
    ctx.fillStyle = 'black';
    ctx.fillRect(0, 0, canvas.width, canvas.height);

    // Draw ball
    ctx.fillStyle = 'white';
    ctx.beginPath();
    ctx.arc(bxLoc, byLoc, 30, 0, Math.PI * 2);
    ctx.fill();

    // Draw paddle
    ctx.fillRect(pxLoc, pyLoc, 150, 20);

    // Draw score
    ctx.font = '70px Arial';
    ctx.fillText(`${score}`, 250, 75);

    requestAnimationFrame(gameLoop);
}

function resetBall() {
    bxLoc = 220;
    byLoc = 30;
    bxVel = Math.random() * 20 - 10;
    byVel = 10;
}

// Mouse movement for paddle
canvas.addEventListener('mousemove', function (e) {
    const rect = canvas.getBoundingClientRect();
    pxLoc = e.clientX - rect.left - 75; // Center paddle on mouse
});

gameLoop();
