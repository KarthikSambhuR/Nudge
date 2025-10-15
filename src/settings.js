window.addEventListener('DOMContentLoaded', async () => {
    console.log('⚙️ Settings window loaded');

    // Check if the Tauri API is available
    if (!window.__TAURI__) {
        console.error('❌ Tauri API not available');
        document.getElementById('status-message').textContent = 'Error: Tauri API not available';
        document.getElementById('status-message').className = 'error';
        return;
    }

    // Destructure necessary functions from the Tauri API
    const { invoke } = window.__TAURI__.core;
    const { Store } = window.__TAURI__.store;
    const { enable, disable } = window.__TAURI__.autostart;

    // Get DOM elements
    const intervalInput = document.getElementById('interval');
    const durationInput = document.getElementById('duration');
    const autostartInput = document.getElementById('autostart');
    const saveButton = document.getElementById('save-btn');
    const statusMessage = document.getElementById('status-message');

    let settingsStore = null;

    // Initialize the settings store
    async function initStore() {
        try {
            console.log('📦 Initializing store...');
            settingsStore = await Store.load('settings.json');
            console.log('✅ Store initialized');
            return true;
        } catch (error) {
            console.error('❌ Failed to initialize store:', error);
            showStatus('Error initializing settings store', 'error');
            return false;
        }
    }

    // Load settings from the store and populate the input fields
    async function loadSettings() {
        if (!settingsStore) {
            const initialized = await initStore();
            if (!initialized) return;
        }

        try {
            console.log('📖 Loading settings...');
            const interval = await settingsStore.get('intervalMinutes') ?? 20;
            const duration = await settingsStore.get('overlayDurationSeconds') ?? 20;
            const autoStart = await settingsStore.get('autoStart') ?? false;

            console.log('✅ Settings loaded:', { interval, duration, autoStart });
            intervalInput.value = interval;
            durationInput.value = duration;
            autostartInput.checked = autoStart;
        } catch (error) {
            console.error('❌ Failed to load settings:', error);
            showStatus('Error loading settings: ' + error.message, 'error');
        }
    }

    // Save the current settings from the input fields to the store
    async function saveSettings() {
        console.log('💾 Saving settings...');
        if (!settingsStore) {
            const initialized = await initStore();
            if (!initialized) return;
        }

        try {
            const interval = parseInt(intervalInput.value, 10);
            const duration = parseInt(durationInput.value, 10);

            // Validate input values
            if (isNaN(interval) || interval < 1 || interval > 180) {
                showStatus('Interval must be between 1-180 minutes', 'error');
                return;
            }
            if (isNaN(duration) || duration < 5 || duration > 300) {
                showStatus('Duration must be between 5-300 seconds', 'error');
                return;
            }

            // Set values in the store
            await settingsStore.set('intervalMinutes', interval);
            await settingsStore.set('overlayDurationSeconds', duration);
            await settingsStore.set('autoStart', autostartInput.checked);
            await settingsStore.save();
            console.log('✅ Settings saved to store');

            // Enable or disable autostart based on the checkbox
            try {
                if (autostartInput.checked) {
                    await enable();
                    console.log('✅ Autostart enabled');
                } else {
                    await disable();
                    console.log('✅ Autostart disabled');
                }
            } catch (error) {
                console.warn('⚠️ Autostart setting failed:', error);
            }

            // Restart the timer in the backend to apply new settings
            console.log('🔄 Restarting timer...');
            await invoke('restart_timer');
            console.log('✅ Timer restarted');

            showStatus('Settings saved successfully!', 'success');
        } catch (error) {
            console.error('❌ Failed to save settings:', error);
            showStatus('Error saving settings: ' + error.message, 'error');
        }
    }

    // Display a status message to the user for 3 seconds
    function showStatus(message, type) {
        statusMessage.textContent = message;
        statusMessage.className = type;
        setTimeout(() => {
            statusMessage.textContent = '';
            statusMessage.className = '';
        }, 3000);
    }

    // --- Event Listeners ---

    // The event listeners for 'input' and 'change' that caused auto-saving have been removed.

    // Add a click event listener to the save button
    saveButton.addEventListener('click', saveSettings);

    // Load settings when the page is ready
    await loadSettings();
});