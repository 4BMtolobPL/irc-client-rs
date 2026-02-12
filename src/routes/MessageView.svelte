<script lang="ts">
    import {currentChannelName, currentServerId, servers} from "../stores/stores.svelte.js";
    import type {ChatMessage} from "../types/irc_types.svelte";

    let container: HTMLDivElement;
    let autoScroll = true;

    let messages = $derived.by((): ChatMessage[] => {
        if (!$currentServerId) return [];
        if (!$currentChannelName) return [];

        return $servers.get($currentServerId)?.channels.get($currentChannelName)?.messages ?? []
    });

    const onScroll = () => {
        autoScroll = container.scrollHeight - container.scrollTop - container.clientHeight < 50;
    }

    $effect(() => {
        if (autoScroll) {
            container.scrollTop = container.scrollHeight;
        }
    });
</script>


<div bind:this={container} class="flex-1 overflow-y-auto p-3" onscroll={onScroll}>
    {#each messages as msg}
        <div class="mb-1">
            <span class="font-semibold">{msg.from}</span>
            <span class="ml-1">{msg.message}</span>
        </div>
    {/each}
</div>