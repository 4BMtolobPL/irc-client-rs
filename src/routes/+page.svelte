<script lang="ts">
    import {listen} from "@tauri-apps/api/event";
    import {invoke} from "@tauri-apps/api/core";

    type IrcMessage = {
        from: string;
        to: string;
        message: string;
    }

    let ircMsgList = $state<IrcMessage[]>([]);
    let msgInput = $state<string>("");

    listen<IrcMessage>("irc:message", (event) => {
        ircMsgList.push(event.payload)
    });

    const sendMessage = async (): Promise<void> => {
        await invoke("send_message", {channel: "#test", message: msgInput});
    }

    $effect(() => {
        to_dark_mode();
    })

    const to_dark_mode = () => {
        console.log("to_dark_mode");
        // onMount 또는 설정 변경 시
        // document.documentElement.classList.toggle("dark", isDarkMode);
        document.documentElement.classList.toggle("dark", true);
    };
</script>

<div class="w-dvw h-dvh flex bg-neutral-100 text-neutral-900
            dark:bg-neutral-900 dark:text-neutral-100">
    <!-- 좌측 채널 목록 -->
    <nav class="w-56 shrink-0 border-r border-neutral-300
                dark:border-neutral-700
                bg-neutral-50 dark:bg-neutral-800
                p-3 overflow-y-auto">
        <ul class="space-y-1 text-sm">
            <li class="px-2 py-1 rounded
                       bg-neutral-200 dark:bg-neutral-700">
                # general
            </li>
            <li class="px-2 py-1 rounded
                       hover:bg-neutral-200 dark:hover:bg-neutral-700">
                # rust
            </li>
            <li class="px-2 py-1 rounded
                       hover:bg-neutral-200 dark:hover:bg-neutral-700">
                # tauri
            </li>
        </ul>
    </nav>

    <!-- 우측 메인 영역 -->
    <main class="flex flex-col flex-1">
        <!-- 메시지 목록 -->
        <section class="flex-1 overflow-y-auto p-4">
            <ul class="space-y-1 text-sm">
                {#each ircMsgList as ircMessage}
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
                {/each}
            </ul>
        </section>

        <!-- 입력 영역 -->
        <section class="border-t border-neutral-300
                        dark:border-neutral-700
                        bg-white dark:bg-neutral-900
                        p-3">
            <form class="flex gap-2" onsubmit={sendMessage}>
                <input
                        type="text"
                        bind:value={msgInput}
                        placeholder="메시지 입력"
                        class="flex-1 rounded px-3 py-2 text-sm
                           bg-white dark:bg-neutral-800
                           border border-neutral-300 dark:border-neutral-700
                           focus:outline-none
                           focus:ring-1 focus:ring-sky-500"
                />
                <button
                        type="submit"
                        class="rounded px-4 py-2 text-sm
                           bg-sky-600 text-white
                           hover:bg-sky-500
                           active:bg-sky-700"
                >
                    Send
                </button>
            </form>
        </section>
    </main>
</div>


<style>
</style>