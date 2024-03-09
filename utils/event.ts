import * as anchor from "@coral-xyz/anchor";

export class EventManager {
    subscriptions: [anchor.Program<any>, number][];

    constructor() {
        this.subscriptions = [];
    }

    subscribe = (program: anchor.Program<any>, eventName: string) => {
        const subscription = program.addEventListener(eventName, event => {
            console.log(`<Event: ${eventName}>`, event);
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
