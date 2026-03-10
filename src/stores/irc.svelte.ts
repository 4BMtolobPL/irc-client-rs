import { SvelteMap } from "svelte/reactivity";
import type { Server } from "../types/kirc.svelte";

export class IrcStore {
  servers = $state(new SvelteMap<string, Server>());
  currentServerId = $state<string | null>(null);
  currentChannelName = $state<string | null>(null);

  currentServer = $derived.by(() => {
    if (!this.currentServerId) return null;
    return this.servers.get(this.currentServerId) ?? null;
  });

  currentChannel = $derived.by(() => {
    if (!this.currentServerId || !this.currentChannelName) return null;
    return this.servers.get(this.currentServerId)?.channels.get(this.currentChannelName) ?? null;
  });

  currentServerNickname = $derived.by(() => {
    return this.currentServer?.nickname ?? null;
  });

  isLocked = $derived.by(() => {
    return this.currentChannel?.locked ?? true;
  });

  serverUnread = $derived.by(() => {
    const result = new Map<string, number>();
    for (const [serverId, server] of this.servers) {
      let total = 0;
      for (const channel of server.channels.values()) {
        total += channel.unread;
      }
      result.set(serverId, total);
    }
    return result;
  });

  constructor() {}
}

export const ircStore = new IrcStore();
