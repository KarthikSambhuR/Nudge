// main.js - Fixed version with correct Tauri API access

window.addEventListener('DOMContentLoaded', async () => {
    console.log('🎬 Overlay window loaded');
    
    const countdownEl = document.getElementById('countdown');
    const progressEl = document.getElementById('progress');
    const skipBtn = document.getElementById('skip-btn');

    // Verify all elements exist
    if (!countdownEl || !progressEl || !skipBtn) {
        console.error('❌ Missing required DOM elements:', {
            countdown: !!countdownEl,
            progress: !!progressEl,
            skipBtn: !!skipBtn
        });
        return;
    }

    let duration = 20; // default
    let timeLeft = duration;
    let intervalId = null;

    // Get the duration from backend
    async function init() {
        try {
            // Check if Tauri API is available
            if (!window.__TAURI__ || !window.__TAURI__.core) {
                console.error('❌ Tauri API not available');
                startCountdown(); // Start with default
                return;
            }

            const invoke = window.__TAURI__.core.invoke;
            console.log('📞 Calling get_overlay_duration...');
            
            duration = await invoke('get_overlay_duration');
            console.log('✅ Got duration:', duration);
            
            timeLeft = duration;
            countdownEl.textContent = timeLeft;
            progressEl.style.width = '100%';
            startCountdown();
        } catch (error) {
            console.error('❌ Failed to get duration:', error);
            // Fallback to default duration if invoke fails
            startCountdown();
        }
    }

    function startCountdown() {
        console.log('⏱️ Starting countdown:', timeLeft, 'seconds');
        
        intervalId = setInterval(() => {
            timeLeft--;
            countdownEl.textContent = timeLeft;
            
            const progress = (timeLeft / duration) * 100;
            progressEl.style.width = progress + '%';

            if (timeLeft <= 0) {
                clearInterval(intervalId);
                closeWindow();
            }
        }, 1000);
    }

    function closeWindow() {
        try {
            if (!window.__TAURI__ || !window.__TAURI__.window) {
                console.error('❌ Tauri window API not available for closing');
                return;
            }
            
            const getCurrentWindow = window.__TAURI__.window.getCurrentWindow;
            const appWindow = getCurrentWindow();
            console.log('🚪 Closing overlay window');
            appWindow.close();
        } catch (error) {
            console.error('❌ Failed to close window:', error);
        }
    }

    skipBtn.addEventListener('click', () => {
        console.log('⏭️ Skip button clicked');
        if (intervalId) clearInterval(intervalId);
        closeWindow();
    });

    // Initialize on load
    await init();
});