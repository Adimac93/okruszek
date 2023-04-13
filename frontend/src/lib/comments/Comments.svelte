<script lang="ts">
    import { onMount } from "svelte";
    import { json } from "../api";
    import type { Comment as CommentData } from "../interfaces";
    import CommentInput from "./CommentInput.svelte";
    import Comment from "./Comment.svelte";
    export let productId: string;

    let comments: Array<CommentData> = [];
    onMount(async () => {
        await getComments();
    });
    async function getComments() {
        const res = await json(`/api/products/${productId}/comments`, "GET");
        if (res) {
            const body = await res.json();
            comments = body as Array<CommentData>;
        }
    }
</script>

{#each comments as comment}
    <Comment author={comment.author} content={comment.content} />
{/each}

<CommentInput bind:productId />
