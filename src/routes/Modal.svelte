<script lang="ts">
    import type {Snippet} from "svelte";

    interface Props {
        dialog: HTMLDialogElement,
        header: Snippet,
        children: Snippet
    }

    let {dialog = $bindable(), header, children}: Props = $props();
</script>

<dialog bind:this={dialog} class="m-auto rounded-md" onclick={(e) => {if (e.target === dialog) dialog.close()}}>
    <div class="w-80 rounded bg-white dark:bg-neutral-800 p-4 shadow-lg">
        {@render header?.()}
        {@render children?.()}
    </div>
</dialog>