<script lang="ts">
    import { json } from "../api";
    import type { Rate } from "../interfaces";
    import { products } from "../stores";

    let rating = 5;

    export let productId: string;

    const rate = async () => {
        const res = await json<Rate>(
            `/api/products/${productId}/ratings`,
            "PUT",
            {
                rating,
            }
        );
        if (res) {
            products.update((products) => {
                const product = products.get(productId);
                product.rating = rating;
                products.set(productId, product);
                return products;
            });
            console.debug("Product rated");
        }
    };
</script>

<div>
    <h1>{rating}/10 🥐</h1>
    <input type="range" min="0" max="10" step="1" bind:value={rating} />
    <button on:click={rate}>Oceń</button>
</div>
