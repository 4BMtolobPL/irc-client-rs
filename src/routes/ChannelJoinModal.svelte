<script lang="ts">
    import {currentChannel, currentServerId, servers} from "../stores/irc.svelte";

    type ChannelJoinForm = {
        name: string;
    };

    let form = $state<ChannelJoinForm>({ name: "" });
    let error = $state<string | null>(null);

    function validate() {
        if (!form.name) {
            error = "채널 이름을 입력하세요";
            return false;
        }
        if (!form.name.startsWith("#")) {
            error = "#으로 시작해야 합니다";
            return false;
        }
        error = null;
        return true;
    }

    function submit() {
        if (!validate()) return;

        const serverId = $currentServerId;
        if (!serverId) return;

        servers.update((map) => {
            const server = map.get(serverId);
            if (!server) return map;

            if (!server.channels.has(form.name)) {
                server.channels.set(form.name, {
                    name: form.name,
                    messages: [],
                    unread: 0,
                });
            }

            return map;
        });

        currentChannel.set(form.name);
        close();
    }

    function close() {
        // 부모에서 모달 닫기
    }
</script>

<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/30">
    <div class="w-full max-w-sm rounded-xl bg-white p-5 shadow-lg dark:bg-neutral-900">
        <header class="mb-4 text-lg font-semibold">채널 추가</header>

        <form class="space-y-3" onsubmit={(e) => {e.preventDefault(); submit();}}>
            <input
                    class="w-full rounded-md border px-3 py-2 dark:bg-neutral-800"
                    placeholder="#channel"
                    bind:value={form.name}
                    autofocus
            />

            {#if error}
                <p class="text-sm text-red-500">{error}</p>
            {/if}

            <footer class="flex justify-end gap-2 pt-3">
                <button
                        type="button"
                        class="rounded-md px-3 py-1.5 text-sm hover:bg-neutral-100 dark:hover:bg-neutral-800"
                        onclick={close}
                >
                    취소
                </button>
                <button
                        type="submit"
                        class="rounded-md bg-blue-600 px-3 py-1.5 text-sm text-white hover:bg-blue-700"
                >
                    추가
                </button>
            </footer>
        </form>
    </div>
</div>