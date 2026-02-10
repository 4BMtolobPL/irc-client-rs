<script lang="ts">
    import Modal from "./Modal.svelte";

    interface Props {
        showDialog: boolean;
    }

    let {showDialog = $bindable()}: Props = $props();

    let newChannelName = $state<string>("");

    const submitAddChannel = () => {
        const name = newChannelName.trim().replace(/^#/, "");
        if (!name) return;

        addChannel(name);   // 기존 로직 재사용
        closeAddChannel();
    }

    const closeAddChannel = () => {
        showDialog.close();
        newChannelName = "";
    }

    const addChannel = (name: string) => {
        channels.update((map) => {
            if (!map.has(name)) {
                map.set(name, {name, messages: []});
            }

            return map;
        });

        currentChannel.set(name);
    }
</script>

<Modal bind:dialog={showDialog}>
    {#snippet header()}
        <h2 class="text-sm font-semibold mb-3">채널 추가</h2>
    {/snippet}
    <input
            bind:value={newChannelName}
            class="w-full rounded px-3 py-2 text-sm
                       border border-neutral-300 dark:border-neutral-700
                       bg-white dark:bg-neutral-900
                       focus:outline-none
                       focus:ring-1 focus:ring-sky-500"
            onkeydown={(e) => {
                    if (e.key === "Enter") submitAddChannel();
                    if (e.key === "Escape") closeAddChannel();
                }}
            placeholder="# channel-name"
            type="text"
    />
    <div class="flex justify-end gap-2 mt-4">
        <button
                class="px-3 py-1 text-sm rounded
                           hover:bg-neutral-100 dark:hover:bg-neutral-700"
                onclick={closeAddChannel}
        >
            취소
        </button>
        <button
                class="px-3 py-1 text-sm rounded
                           bg-sky-600 text-white
                           hover:bg-sky-500"
                onclick={submitAddChannel}
        >
            추가
        </button>
    </div>
</Modal>
