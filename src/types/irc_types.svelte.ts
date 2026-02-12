export type ChatMessage = | {
    type: "user"; id: string; nickname: string; content: string; timestamp: number;
} | {
    type: "system"; id: string; content: string; timestamp: number;
}

export type Channel = {
    name: string; topic?: string; messages: ChatMessage[]; users: Set<string>; unread: number;
}

export type Server = {
    id: string; name: string; host: string; port: number; tls: boolean; nickname: string; status: IrcServerStatus;

    channels: Map<string, Channel>; serverMessages: ChatMessage[];
}

// TODO: enum으로 바꿀수 있지 않을까?
export type IrcServerStatus = | "connecting" | "connected" | "disconnected" | "error";
