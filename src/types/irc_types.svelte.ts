export type IrcMessage = {
    from: string; message: string; timestamp: number;
}

export type IrcChannel = {
    name: string; messages: IrcMessage[]; unread: number;
}

export type IrcServerState = {
    id: string;
    name: string;
    host: string;
    port: number;
    tls: boolean;
    nickname: string;
    status: IrcServerStatus;
    channels: Map<string, IrcChannel>;
}

// TODO: enum으로 바꿀수 있지 않을까?
export type IrcServerStatus = | "connecting" | "connected" | "disconnected" | "error";
