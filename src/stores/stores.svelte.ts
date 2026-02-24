import type {ChatMessage, Server} from "../types/irc_types.svelte";
import {SvelteMap, SvelteSet} from "svelte/reactivity";
import {derived, writable} from "svelte/store";

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
        return newMap;
    });
}

export const addUnreadMessage = (serverId: string, channelName: string, unread: number) => {
    servers.update((map) => {
        const newMap = new SvelteMap(map);
        const server = newMap.get(serverId);
        if (!server) return newMap;

        const channel = server.channels.get(channelName);
        if (!channel) return newMap;

        channel.unread += unread;
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
