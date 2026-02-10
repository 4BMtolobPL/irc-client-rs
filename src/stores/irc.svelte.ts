import {writable} from "svelte/store";
import type {IrcServerState} from "../types/irc_types.svelte";

export const servers = writable<Map<string, IrcServerState>>(new Map());
export const currentServerId = writable<string | null>(null);
export const currentChannel = writable<string | null>(null);