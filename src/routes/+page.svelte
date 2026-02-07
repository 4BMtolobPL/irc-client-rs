<script lang="ts">
    import {listen} from "@tauri-apps/api/event";
    import {invoke} from "@tauri-apps/api/core";

    let ircMsgList = $state<string[]>([]);
    let msgInput = $state<string>("");

    listen<[string, string]>("irc:message", (event) => {
        ircMsgList.push(event.payload[1])
    });

    const sendMessage = async (): Promise<void> => {
        await invoke("send_message");
    }
</script>

<main class="container">
    <section>
        <ul>
            {#each ircMsgList as ircMessage}
                <li>{ircMessage}</li>
            {/each}
        </ul>
    </section>
    <section>
        <input type="text" bind:value={msgInput} />
        <button onclick={sendMessage}>Send</button>
    </section>
</main>

<style>
</style>
