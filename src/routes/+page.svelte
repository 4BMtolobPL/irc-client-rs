<script lang="ts">
    import {listen} from "@tauri-apps/api/event";
    import {invoke} from "@tauri-apps/api/core";
    import {derived, writable} from "svelte/store";

    type IrcMessage = {
        from: string;
        to: string;
        message: string;
    }

    type Channel = {
        name: string;
        messages: IrcMessage[];
    }

    type ServerState = {
        id: string;
        channels: Map<string, Channel>;
    }

    const currentChannel = writable<string | null>(null);
    const channels = writable<Map<string, Channel>>(new Map());
    const currentMessages = derived(
        [channels, currentChannel],
        ([$channels, $currentChannel]) =>
            $currentChannel ? $channels.get($currentChannel)?.messages ?? [] : []
    );

    let dialog = $state<HTMLDialogElement>()!;
    let ircMsgList = $state<IrcMessage[]>([]);
    let msgInput = $state<string>("");
    let newChannelName = $state<string>("");
    let scrollAnchor: HTMLDivElement;


    listen<IrcMessage>("irc:message", (event) => {
        ircMsgList.push(event.payload)
    });

    $effect(() => {
        scrollAnchor?.scrollIntoView({behavior: "smooth"});
    })

    const sendMessage = async (): Promise<void> => {
        await invoke("send_message", {channel: "#test", message: msgInput});
    }

    const openAddChannel = () => {
        dialog.showModal();
    }

    const closeAddChannel = () => {
        dialog.close();
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

    const submitAddChannel = () => {
        const name = newChannelName.trim().replace(/^#/, "");
        if (!name) return;

        addChannel(name);   // 기존 로직 재사용
        closeAddChannel();
    }

    const onIrcMessage = (channelName: string, msg: IrcMessage) => {
        channels.update((map) => {
            const channel = map.get(channelName);
            if (channel) {
                channel.messages.push(msg);
            }

            return map;
        });
    }
</script>

<div class="w-dvw h-dvh flex bg-neutral-100 text-neutral-900
            dark:bg-neutral-900 dark:text-neutral-100">
    <!-- 좌측 채널 목록 -->
    <nav class="w-56 shrink-0 border-r border-neutral-300
                dark:border-neutral-700
                bg-neutral-50 dark:bg-neutral-800
                p-3 flex flex-col">

        <!-- 채널 헤더 -->
        <div class="flex items-center justify-between mb-3">
            <span class="text-sm font-semibold tracking-wide">
                Channels
            </span>
            <button
                    class="w-6 h-6 flex items-center justify-center
                       rounded
                       hover:bg-neutral-200 dark:hover:bg-neutral-700
                       text-neutral-600 dark:text-neutral-300"
                    onclick={openAddChannel}
                    title="채널 추가"
                    type="button"
            >
                +
            </button>
        </div>

        <!-- 채널 목록 -->
        <ul class="flex-1 space-y-1 text-sm overflow-y-auto">
            {#each Array.from($channels.values()) as channel}
                <li class="px-2 py-1 rounded cursor-pointer {channel.name === $currentChannel ? 'bg-neutral-200 dark:bg-neutral-700' : 'hover:bg-neutral-200 dark:hover:bg-neutral-700'}">
                    <button onclick={() => currentChannel.set(channel.name)}># {channel.name}</button>
                </li>
            {/each}
            <!--<li class="px-2 py-1 rounded
                       bg-neutral-200 dark:bg-neutral-700">
                # general
            </li>
            <li class="px-2 py-1 rounded
                       hover:bg-neutral-200 dark:hover:bg-neutral-700">
                # rust
            </li>
            <li class="px-2 py-1 rounded
                       hover:bg-neutral-200 dark:hover:bg-neutral-700">
                # tauri
            </li>-->
        </ul>
    </nav>

    <!-- 우측 메인 영역 -->
    <main class="flex flex-col flex-1">
        <!-- 메시지 목록 -->
        <section class="flex-1 overflow-y-auto p-4">
            <ul class="space-y-1 text-sm">
                {#if $currentChannel}
                    {#each $currentMessages as msg}
                        <li>
                            <span class="font-semibold">{msg.from}</span>
                            : {msg.message}
                        </li>
                    {/each}
                {/if}
                <!--{#each ircMsgList as ircMessage}
                    <li>
                        <span class="font-semibold text-sky-600
                                     dark:text-sky-400">
                            {ircMessage.from}
                        </span>
                        <span class="text-neutral-700
                                     dark:text-neutral-300">
                            : {ircMessage.message}
                        </span>
                    </li>
                {/each}-->
            </ul>
            <div bind:this={scrollAnchor}></div>
        </section>

        <!-- 입력 영역 -->
        <section class="border-t border-neutral-300
                        dark:border-neutral-700
                        bg-white dark:bg-neutral-900
                        p-3">
            <form class="flex gap-2" onsubmit={sendMessage}>
                <input
                        bind:value={msgInput}
                        class="flex-1 rounded px-3 py-2 text-sm
                           bg-white dark:bg-neutral-800
                           border border-neutral-300 dark:border-neutral-700
                           focus:outline-none
                           focus:ring-1 focus:ring-sky-500"
                        placeholder="메시지 입력"
                        type="text"
                />
                <button
                        class="rounded px-4 py-2 text-sm
                           bg-sky-600 text-white
                           hover:bg-sky-500
                           active:bg-sky-700"
                        type="submit"
                >
                    Send
                </button>
            </form>
        </section>
    </main>
</div>

<dialog bind:this={dialog}
        class="m-auto rounded-md"
        onclick={(e) => {if (e.target === dialog) closeAddChannel()}}>
    <div class="w-80 rounded bg-white dark:bg-neutral-800 p-4 shadow-lg">
        <h2 class="text-sm font-semibold mb-3">
            채널 추가
        </h2>
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
    </div>
</dialog>

<style>
</style>