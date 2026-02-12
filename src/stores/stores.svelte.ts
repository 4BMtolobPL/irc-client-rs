import {derived, writable} from "svelte/store";
import type {ChatMessage, Server} from "../types/irc_types.svelte";

export const servers = writable<Map<string, Server>>(new Map());
export const currentServerId = writable<string | null>(null);
export const currentChannelName = writable<string | null>(null);

export const ensureChannel = (serverId: string, channelName: string) => {
    servers.update((map) => {
        const server = map.get(serverId);
        if (!server) return map;

        if (!server.channels.has(channelName)) {
            server.channels.set(channelName, {
                name: channelName, messages: [], users: new Set(), unread: 0
            });
        }

        return map;
    });
};

export const addMessage = (serverId: string, channelName: string, message: ChatMessage) => {
    servers.update((map) => {
        const server = map.get(serverId);
        if (!server) return map;

        const channel = server.channels.get(channelName);
        if (!channel) return map;

        channel.messages.push(message);

        return map;
    });
}

export const addServerMessage = (serverId: string, message: ChatMessage) => {
    servers.update((map) => {
        const server = map.get(serverId);
        if (!server) return map;

        server.serverMessages.push(message);

        return map;
    });
}

export const currentChannel = derived(
    [servers, currentServerId, currentChannelName],
    ([$servers, $serverId, $channelName]) => {
        if (!$serverId || !$channelName) return null;
        return $servers.get($serverId)?.channels.get($channelName) ?? null;
    }
);