<script lang="ts">
    import { json } from "../api";
    import type { AddComment } from "../interfaces";

    export let productId;

    let content;
    async function addComment() {
        const res = await json<AddComment>(
            `/api/products/${productId}/comments`,
            "PUT",
            {
                content,
            }
        );
    }

    async function keyEvent(e) {
        
    }
</script>

<input type="text" placeholder="Twój komentarz" bind:value={content} />
<button on:click={addComment}>Opublikuj</button>
