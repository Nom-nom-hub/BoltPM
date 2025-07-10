// Smooth scroll for navigation
// Animated particles background

document.querySelectorAll('nav a[href^="#"]').forEach(anchor => {
    anchor.addEventListener('click', function (e) {
        e.preventDefault();
        const target = document.querySelector(this.getAttribute('href'));
        if (target) {
            target.scrollIntoView({ behavior: 'smooth' });
        }
    });
});

// Particle background effect
const canvas = document.getElementById('particles-bg');
if (canvas) {
    const ctx = canvas.getContext('2d');
    let w = window.innerWidth;
    let h = window.innerHeight;
    canvas.width = w;
    canvas.height = h;
    let particles = [];
    const colors = ['#ffd700', '#fffbe6', '#00cfff', '#ffe066'];
    function random(min, max) { return Math.random() * (max - min) + min; }
    function createParticle() {
        return {
            x: random(0, w),
            y: random(0, h),
            r: random(1.5, 3.5),
            color: colors[Math.floor(random(0, colors.length))],
            dx: random(-0.3, 0.3),
            dy: random(0.1, 0.5),
            alpha: random(0.3, 0.8)
        };
    }
    function draw() {
        ctx.clearRect(0, 0, w, h);
        for (let p of particles) {
            ctx.save();
            ctx.globalAlpha = p.alpha;
            ctx.beginPath();
            ctx.arc(p.x, p.y, p.r, 0, 2 * Math.PI);
            ctx.fillStyle = p.color;
            ctx.shadowColor = p.color;
            ctx.shadowBlur = 12;
            ctx.fill();
            ctx.restore();
        }
    }
    function update() {
        for (let p of particles) {
            p.x += p.dx;
            p.y += p.dy;
            if (p.y > h + 10) {
                p.x = random(0, w);
                p.y = -10;
            }
            if (p.x < -10 || p.x > w + 10) {
                p.x = random(0, w);
                p.y = -10;
            }
        }
    }
    function loop() {
        draw();
        update();
        requestAnimationFrame(loop);
    }
    function resize() {
        w = window.innerWidth;
        h = window.innerHeight;
        canvas.width = w;
        canvas.height = h;
    }
    window.addEventListener('resize', resize);
    particles = Array.from({length: 60}, createParticle);
    loop();
}
