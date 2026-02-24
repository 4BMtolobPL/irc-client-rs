import type {ChatMessage, Server} from "../types/irc_types.svelte";
import {SvelteMap, SvelteSet} from "svelte/reactivity";
import {derived, get, writable} from "svelte/store";

export const servers = writable<SvelteMap<string, Server>>(new SvelteMap());
export const currentServerId = writable<string | null>(null);
export const currentChannelName = writable<string | null>(null);


export const ensureChannel = (serverId: string, channelName: string) => {
    servers.update((map) => {
        const newMap = new SvelteMap(map);
        const server = newMap.get(serverId);
        if (!server) return newMap;

        if (!server.channels.has(channelName)) {
            server.channels.set(channelName, {
                name: channelName, messages: [], users: new SvelteSet(), unread: 0,
            });
        }

        return newMap;
    });
}

export const addMessage = (serverId: string, channelName: string, message: ChatMessage) => {
    servers.update((map) => {
        const newMap = new SvelteMap(map);
        const server = newMap.get(serverId);
        if (!server) return newMap;

        const channel = server.channels.get(channelName);
        if (!channel) return newMap;

        channel.messages = [...channel.messages, message];

        // unread 처리
        const isCurrent = get(currentServerId) === serverId && get(currentChannelName) === channelName;
        if (!isCurrent && message.type === "user") {
            channel.unread += 1;
        }

        return newMap;
    });
}

export const removeUnreadMessage = (serverId: string, channelName: string) => {
    servers.update((map) => {
        const newMap = new SvelteMap(map);
        const server = newMap.get(serverId);
        if (!server) return newMap;

        const channel = server.channels.get(channelName);
        if (!channel) return newMap;

        channel.unread = 0;
        return newMap;
    });
}

export const addServerMessage = (serverId: string, message: ChatMessage) => {
    servers.update((map) => {
        const newMap = new SvelteMap(map);
        const server = newMap.get(serverId);
        if (!server) return newMap;

        server.serverMessages = [...server.serverMessages, message,];

        return newMap;
    });
}

export const currentServerNickname = derived([servers, currentServerId], ([$servers, $serverId]): string | null => {
    if (!$serverId) return null;
    return $servers.get($serverId)?.nickname ?? null;
});

export const currentChannel = derived([servers, currentServerId, currentChannelName], ([$servers, $serverId, $channelName]) => {
    if (!$serverId || !$channelName) return null;
    return $servers.get($serverId)?.channels.get($channelName) ?? null;
});

export const serverUnread = derived(servers, ($servers) => {
    const result = new Map<string, number>();

    for (const [serverId, server] of $servers) {
        let total = 0;
        for (const channel of server.channels.values()) {
            total += channel.unread;
        }
        result.set(serverId, total);
    }

    return result;
});