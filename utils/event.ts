import * as anchor from "@coral-xyz/anchor";

export class EventManager {
    subscriptions: [anchor.Program<any>, number][];
    callback: (eventName: string, event: unknown) => void;

    constructor(callback: (eventName: string, event: unknown) => void) {
        this.subscriptions = [];
        this.callback = callback;
    }

    subscribe = (program: anchor.Program<any>, eventName: string) => {
        const subscription = program.addEventListener(eventName, event => {
            this.callback(eventName, event);
        });
        this.subscriptions.push([
            program,
            subscription,
        ]);
    };

    unsubscribeAll = () => {
        for (let index = 0; index < this.subscriptions.length; index++) {
            const [program, subscription] = this.subscriptions[index];
            program.removeEventListener(subscription);
        }
    };
}
