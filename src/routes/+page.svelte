<script lang="ts">
    import {listen} from "@tauri-apps/api/event";
    import {invoke} from "@tauri-apps/api/core";

    type IrcMessage = {
        from: string;
        to: string;
        message: string;
    }

    let ircMsgList = $state<IrcMessage[]>([]);
    let msgInput = $state<string>("");

    listen<IrcMessage>("irc:message", (event) => {
        ircMsgList.push(event.payload)
    });

    const sendMessage = async (): Promise<void> => {
        await invoke("send_message", {channel: "#test", message: msgInput});
    }
</script>

<main class="container">
    <section class="receive-message">
        <ul>
            {#each ircMsgList as ircMessage}
                <li>{ircMessage.from}: {ircMessage.message}</li>
            {/each}
        </ul>
    </section>
    <section class="send-message">
        <form onsubmit={sendMessage}>
            <input type="text" bind:value={msgInput} />
            <button type="submit">Send</button>
        </form>
    </section>
</main>

<style>
    .container {
        /*background-color: antiquewhite;*/
        width: 95vw;
        height: 95vh;
        display: flex;
        flex-direction: column;
    }

    .receive-message {
        /*background-color: green;*/
        flex-grow: 1;
        overflow-x: hidden;
        overflow-y: auto;
    }

    .send-message {
        /*background-color: blue;*/
        flex-basis: 2rem;
    }

    .send-message form {
        display: flex;
    }

    .send-message form input {
        flex-grow: 1;
    }

    .send-message form button {
        flex-grow: 0;
    }
</style>