let isInitialized: Promise<void>;
let resolveInitialized: () => void;
let initialized = false;

isInitialized = new Promise<void>((resolve) => {
    resolveInitialized = resolve;
});

export const setInitialized = () => {
    if (!initialized) {
        resolveInitialized();
        initialized = true;
    }
}

export const waitForSetup = async () => {
    if (!initialized) {
        await isInitialized;
    }
}
