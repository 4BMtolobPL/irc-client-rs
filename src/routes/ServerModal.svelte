<script lang="ts">
    interface Props {
        showServerModal: boolean;
    }

    let { showServerModal = $bindable() }: Props = $props();

    type ServerConnectForm = {
        host: string;
        port: number;
        tls: boolean;
        nickname: string;
    };

    let form = $state<ServerConnectForm>({
        host: "",
        port: 6697,
        tls: true,
        nickname: "",
    });

    let errors = $state<Record<string, string>>({});
    let connecting = $state(false);

    const validate = () => {
        const e: Record<string, string> = {};

        if (!form.host) e.host = "서버 주소를 입력하세요";
        if (!form.nickname) e.nickname = "닉네임을 입력하세요";
        if (!form.port || form.port <= 0) e.port = "유효한 포트를 입력하세요";

        errors = e;
        return Object.keys(e).length === 0;
    }

    const submit = () => {
        if (!validate()) return;
        connecting = true;

        // 여기서 parent로 connect 이벤트 emit
    }

    const cancel = () => {
        if (connecting) return;
        // 모달 닫기
        showServerModal = false;
    }
</script>

<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/30">
    <div class="w-full max-w-md rounded-xl bg-white p-6 shadow-lg dark:bg-neutral-900">
        <!-- Header -->
        <header class="mb-4 flex items-center justify-between">
            <h2 class="text-lg font-semibold">서버 연결</h2>
            <button class="text-neutral-500 hover:text-neutral-800 dark:hover:text-neutral-200" onclick={cancel}>✕</button>
        </header>

        <!-- Form -->
        <form class="space-y-4" onsubmit={(e) => {e.preventDefault(); submit();}}>
            <!-- Host -->
            <div>
                <label class="block text-sm font-medium mb-1">
                    서버 주소
                <input class="w-full rounded-md border px-3 py-2 dark:bg-neutral-800" bind:value={form.host}/>
                </label>
                {#if errors.host}
                    <p class="text-sm text-red-500">{errors.host}</p>
                {/if}
            </div>

            <!-- Port + TLS -->
            <div class="flex gap-3">
                <div class="flex-1">
                    <label class="block text-sm font-medium mb-1">
                        포트
                    <input
                            type="number"
                            class="w-full rounded-md border px-3 py-2
                               dark:bg-neutral-800"
                            bind:value={form.port}
                    />
                    </label>
                </div>

                <div class="flex items-end">
                    <label class="flex items-center gap-2 text-sm">
                        <input type="checkbox" bind:checked={form.tls} />
                        TLS
                    </label>
                </div>
            </div>

            <!-- Nickname -->
            <div>
                <label class="block text-sm font-medium mb-1">
                    닉네임
                <input
                        class="w-full rounded-md border px-3 py-2
                           dark:bg-neutral-800"
                        bind:value={form.nickname}
                />
                </label>
                {#if errors.nickname}
                    <p class="text-sm text-red-500">{errors.nickname}</p>
                {/if}
            </div>

            <!-- Actions -->
            <footer class="flex justify-end gap-2 pt-4">
                <button
                        type="button"
                        class="rounded-md px-4 py-2 text-sm
                           hover:bg-neutral-100
                           dark:hover:bg-neutral-800"
                        onclick={cancel}
                >
                    취소
                </button>

                <button
                        type="submit"
                        disabled={connecting}
                        class="rounded-md bg-blue-600 px-4 py-2 text-sm text-white
                           hover:bg-blue-700 disabled:opacity-50"
                >
                    {connecting ? "연결 중..." : "연결"}
                </button>
            </footer>
        </form>
    </div>
</div>