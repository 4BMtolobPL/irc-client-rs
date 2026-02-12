<script lang="ts">
    import {listen} from "@tauri-apps/api/event";
    import {invoke} from "@tauri-apps/api/core";
    import type {IrcMessage} from "../types/irc_types.svelte.js";
    import ServerModal from "./ServerModal.svelte";
    import {currentChannel, currentServerId, servers} from "../stores/stores.svelte.js";
    import ChannelJoinModal from "./ChannelJoinModal.svelte";
    import MessageView from "./MessageView.svelte";

    let showChannelModal = $state<boolean>(false);
    let msgInput = $state<string>("");

    let showServerModal = $state<boolean>(false);

    type Payload = {
        serverId: string;
        channel: string;
        from: string;
        message: string;
        timestamp: number;
    }

    listen<Payload>("kirc:message", (event) => {
        const {serverId, channel, from, message, timestamp} = event.payload;
        onIrcMessage(serverId, channel, {from, message, timestamp});
    });

    const sendMessage = async (): Promise<void> => {
        await invoke("send_message", {channel: "#test", message: msgInput});
    }

    const onIrcMessage = (serverId: string, channelName: string, msg: IrcMessage) => {
        servers.update((map) => {
            const server = map.get(serverId);
            if (!server) return map;

            const channel = server.channels.get(channelName);
            if (!channel) return map;

            channel.messages.push(msg);

            if (
                serverId !== $currentServerId ||
                channelName !== $currentChannel
            ) {
                channel.unread += 1;
            }

            return map;
        });
    }

    const selectServer = (serverId: string) => {
        currentServerId.set(serverId);
        currentChannel.set(null);
    }

    const selectChannel = (name: string) => {
        const serverId = $currentServerId;
        if (!serverId) return;

        servers.update((map) => {
            const server = map.get(serverId);
            if (!server) return map;

            const channel = server.channels.get(name);
            if (channel) {
                channel.unread = 0;
            }

            return map;
        });

        currentChannel.set(name);
    }

    const openChannelModal = () => {
        showChannelModal = true;
    }

    const openServerModal = () => {
        showServerModal = true;
    }
</script>

<div class="w-dvw h-dvh flex bg-neutral-100 text-neutral-900 dark:bg-neutral-900 dark:text-neutral-100">
    <!-- 좌측 패널 HTML 구조 -->
    <aside class="w-56 shrink-0 border-r bg-neutral-50 dark:bg-neutral-900">
        <div class="p-2 text-sm font-semibold">Servers</div>
        <ul class="space-y-1 px-2">
            {#each Array.from($servers.values()) as server}
                <li>
                    <!-- Server Row -->
                    <button class="w-full flex items-center justify-between rounded px-2 py-1 {server.id === $currentServerId ? 'bg-neutral-200 dark:bg-neutral-700' : 'hover:bg-neutral-200 dark:hover:bg-neutral-700'}"
                            onclick={() => selectServer(server.id)}>
                        <span class="truncate">{server.name}</span>
                        <!-- Status Dot -->
                        <span class="h-2 w-2 rounded-full {server.status === 'connected' ? 'bg-green-500' : server.status === 'connecting' ? 'bg-yellow-500' : 'bg-red-500'}"></span>
                    </button>

                    <!-- Channel List -->
                    {#if server.id === $currentServerId}
                        <ul class="ml-4 mt-1 space-y-1 text-sm">
                            {#each Array.from(server.channels.values()) as channel}
                                <li class="cursor-pointer rounded px-2 py-1 {channel.name === $currentChannel ? 'bg-neutral-300 dark:bg-neutral-600' : 'hover:bg-neutral-200 dark:hover:bg-neutral-700'}">
                                    <button onclick={() => currentChannel.set(channel.name)}>
                                        <span class="flex items-center gap-1">
                                            # {channel.name}
                                            {#if channel.unread > 0}
                                                <span class="rounded-full bg-red-500 px-1.5 text-xs text-white">
                                                    {channel.unread}
                                                </span>
                                            {/if}
                                        </span>
                                    </button>
                                </li>
                            {/each}

                            <!-- Channel Add -->
                            <li>
                                <button class="w-full cursor-pointer flex items-center justify-between rounded px-2 py-1 text-neutral-500 hover:bg-neutral-200 dark:hover:bg-neutral-700"
                                        onclick={() => openChannelModal()}>+ 채널 추가
                                </button>
                            </li>
                        </ul>
                    {/if}
                </li>
            {/each}

            <!-- Server Add -->
            <li>
                <button class="w-full cursor-pointer flex items-center justify-between mt-2 rounded px-2 py-1 text-sm text-neutral-600 hover:bg-neutral-200 dark:text-neutral-400 dark:hover:bg-neutral-700"
                        onclick={() => openServerModal()}>+ 서버 추가
                </button>
            </li>
        </ul>
    </aside>

    <!-- 우측 메인 영역 -->
    <main class="flex flex-col flex-1">
        <!-- 메시지 목록 -->
        <!--<section class="flex-1 overflow-y-auto p-4">
            <ul class="space-y-1 text-sm">
                {#if currentChannel}
                    {#each [] as msg}
                        <li>
                            <span class="font-semibold">{msg.from}</span>: {msg.message}
                        </li>
                    {/each}
                {/if}
                &lt;!&ndash;{#each ircMsgList as ircMessage}
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
                {/each}&ndash;&gt;
            </ul>
        </section>-->
        <MessageView></MessageView>

        <!-- 입력 영역 -->
        <section class="border-t border-neutral-300 dark:border-neutral-700 bg-white dark:bg-neutral-900 p-3">
            <form class="flex gap-2" onsubmit={sendMessage}>
                <input bind:value={msgInput}
                       class="flex-1 rounded px-3 py-2 text-sm bg-white dark:bg-neutral-800 border border-neutral-300 dark:border-neutral-700 focus:outline-none focus:ring-1 focus:ring-sky-500"
                       placeholder="메시지 입력" type="text"/>
                <button class="rounded px-4 py-2 text-sm bg-sky-600 text-white hover:bg-sky-500 active:bg-sky-700"
                        type="submit">Send
                </button>
            </form>
        </section>
    </main>
</div>


<!--<ChannelModal bind:showChannelDialog></ChannelModal>-->
{#if showChannelModal}
    <ChannelJoinModal></ChannelJoinModal>
{/if}
{#if showServerModal}
    <ServerModal bind:showServerModal></ServerModal>
{/if}
<style>
</style>