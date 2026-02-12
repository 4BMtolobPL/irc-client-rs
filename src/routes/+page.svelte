<script lang="ts">
    import {listen} from "@tauri-apps/api/event";
    import {invoke} from "@tauri-apps/api/core";
    import type {ChatMessage} from "../types/irc_types.svelte.js";
    import ServerModal from "./ServerModal.svelte";
    import {
        addMessage,
        addServerMessage,
        currentChannelName,
        currentServerId,
        ensureChannel,
        servers
    } from "../stores/stores.svelte.js";
    import ChannelJoinModal from "./ChannelJoinModal.svelte";
    import MessageView from "./MessageView.svelte";

    let showChannelModal = $state<boolean>(false);
    let msgInput = $state<string>("");

    let showServerModal = $state<boolean>(false);

    /*type Payload = {
        serverId: string;
        channel: string;
        from: string;
        message: string;
        timestamp: number;
    }*/

    /*listen<Payload>("kirc:message", (event) => {
        const {serverId, channel, from, message, timestamp} = event.payload;
        onIrcMessage(serverId, channel, {type: "user", id: serverId, nickname: from, content: message, timestamp});
    });*/

    listen<UiEventPayload>("kirc:event", (event) => {
        const payload: UiEventPayload = event.payload;

        switch (payload.type) {
            case "UserMessage":
                ensureChannel(payload.server_id, payload.channel);
                addMessage(payload.server_id, payload.channel, {
                    type: "user",
                    id: crypto.randomUUID(),
                    nickname: payload.nick,
                    content: payload.content,
                    timestamp: payload.timestamp,
                });
                break;
            case "Join":
                ensureChannel(payload.server_id, payload.channel);

                servers.update((map) => {
                    const server = map.get(payload.server_id);
                    if (!server) return map;

                    const channel = server.channels.get(payload.channel);
                    if (!channel) return map;

                    channel.users.add(payload.nick);

                    channel.messages.push({
                        type: "system",
                        id: crypto.randomUUID(),
                        content: `${payload.nick} joined the channel`,
                        timestamp: Date.now(),
                    });

                    // 🔥 내가 JOIN 했을 경우
                    if (payload.nick === server.nickname) {
                        currentServerId.set(payload.server_id);
                        currentChannelName.set(payload.channel);
                    }

                    return map;
                });

                break;
            case "Part":
                servers.update((map) => {
                    const server = map.get(payload.server_id);
                    if (!server) return map;

                    const channel = server.channels.get(payload.channel);
                    if (!channel) return map;

                    channel.users.delete(payload.nick);

                    channel.messages.push({
                        type: "system",
                        id: crypto.randomUUID(),
                        content: `${payload.nick} left the channel`,
                        timestamp: Date.now(),
                    });

                    // 🔥 내가 나간 경우
                    if (payload.nick === server.nickname) {
                        currentChannelName.set(null);
                    }

                    return map;
                });

                break;
            case "Quit":
                servers.update((map) => {
                    const server = map.get(payload.server_id);
                    if (!server) return map;

                    // 🔥 모든 채널 순회
                    for (const channel of server.channels.values()) {
                        if (channel.users.has(payload.nick)) {
                            channel.users.delete(payload.nick);

                            channel.messages.push({
                                type: "system",
                                id: crypto.randomUUID(),
                                content: `${payload.nick} quit${payload.reason ? ` (${payload.reason})` : ""}`,
                                timestamp: Date.now(),
                            });
                        }
                    }

                    return map;
                });
                break;
            case "Nick":
                servers.update((map) => {
                    const server = map.get(payload.server_id);
                    if (!server) return map;

                    // 🔥 내가 닉네임 변경한 경우
                    if (server.nickname === payload.old_nick) {
                        server.nickname = payload.new_nick;
                    }

                    for (const channel of server.channels.values()) {
                        if (channel.users.has(payload.old_nick)) {
                            channel.users.delete(payload.old_nick);
                            channel.users.add(payload.new_nick);

                            channel.messages.push({
                                type: "system",
                                id: crypto.randomUUID(),
                                content: `${payload.old_nick} is now known as ${payload.new_nick}`,
                                timestamp: Date.now(),
                            });
                        }
                    }

                    return map;
                });

                break;
            case "Topic":
                ensureChannel(payload.server_id, payload.channel);

                servers.update((map) => {
                    const server = map.get(payload.server_id);
                    if (!server) return map;

                    const channel = server.channels.get(payload.channel);
                    if (!channel) return map;

                    channel.topic = payload.topic;

                    channel.messages.push({
                        type: "system",
                        id: crypto.randomUUID(),
                        content: `Topic set to: ${payload.topic}`,
                        timestamp: Date.now(),
                    });

                    return map;
                });


                break;
            case "Error":
                addServerMessage(payload.server_id, {
                    type: "system",
                    id: crypto.randomUUID(),
                    content: `Error: ${payload.message}`,
                    timestamp: Date.now(),
                });
                break;
        }
    });

    const sendMessage = async (): Promise<void> => {
        await invoke("send_message", {channel: "#test", message: msgInput});
    }

    const onIrcMessage = (serverId: string, channelName: string, msg: ChatMessage) => {
        servers.update((map) => {
            const server = map.get(serverId);
            if (!server) return map;

            const channel = server.channels.get(channelName);
            if (!channel) return map;

            channel.messages.push(msg);

            if (
                serverId !== $currentServerId ||
                channelName !== $currentChannelName
            ) {
                channel.unread += 1;
            }

            return map;
        });
    }

    const selectServer = (serverId: string) => {
        currentServerId.set(serverId);
        currentChannelName.set(null);
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

        currentChannelName.set(name);
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
                                <li class="cursor-pointer rounded px-2 py-1 {channel.name === $currentChannelName ? 'bg-neutral-300 dark:bg-neutral-600' : 'hover:bg-neutral-200 dark:hover:bg-neutral-700'}">
                                    <button onclick={() => currentChannelName.set(channel.name)}>
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