class OnlineStore {
    online = $state(true);

    start() {
        const update = () => (this.online = navigator.onLine);
        update();
        window.addEventListener("online", update);
        window.addEventListener("offline", update);
        return () => {
            window.removeEventListener("online", update);
            window.removeEventListener("offline", update);
        };
    }
}

export const connectivity = new OnlineStore();
