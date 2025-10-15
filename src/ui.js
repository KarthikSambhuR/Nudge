// ui.js - Handles UI elements like the local time clock

window.addEventListener('DOMContentLoaded', () => {
    const timeEl = document.getElementById('local-time');

    if (!timeEl) {
        console.error("❌ Time display element not found!");
        return;
    }

    function updateTime() {
        const now = new Date();
        // Format to 12-hour format with AM/PM (e.g., 10:30 PM)
        const timeString = now.toLocaleTimeString('en-US', {
            hour: 'numeric',
            minute: 'numeric',
            hour12: true
        });
        timeEl.textContent = timeString;
    }

    // Update time immediately on load and then every second
    updateTime();
    setInterval(updateTime, 1000);
});