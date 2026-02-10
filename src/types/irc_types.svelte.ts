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

export type IrcServerStatus =
    | "connecting"
    | "connected"
    | "disconnected"
    | "error";
