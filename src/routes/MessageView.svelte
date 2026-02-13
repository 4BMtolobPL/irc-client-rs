<script lang="ts">
    import {currentChannel} from "../stores/stores.svelte";

    let container: HTMLDivElement;
    let autoScroll = true;

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
    {#each $currentChannel?.messages ?? [] as msg}
        {#if msg.type === "user"}
            <div class="mb-1">
                <span class="font-semibold">{msg.nickname}</span>
                <span class="ml-1 whitespace-pre-wrap">{msg.content}</span>
            </div>
        {/if}
        {#if msg.type === "system"}
            <div class="mb-1">
                <span class="font-semibold">System</span>
                <span class="ml-1 whitespace-pre-wrap">{msg.content}</span>
            </div>
        {/if}
    {/each}
</div>