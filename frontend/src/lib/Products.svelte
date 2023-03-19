<script lang="ts">
  import { onMount } from "svelte";
  import { Fetch } from "./api";
  import Rate from "./Rate.svelte";
  import { products, type Product } from "./stores";

  const api = new Fetch();
  onMount(async () => {
    const res = await api.json("/api/products", "GET");
    if (res) {
      const body = await res.json();
      let pr = new Map<string, Product>();
      Object.entries(body).forEach(([id, body]) => {
        pr.set(id, body as Product);
      });
      products.set(pr);
    }
  });

  const loadRatings = async (id: string) => {
    const res = await api.json(`/api/products/ratings/${id}`, "GET");
    if (res) {
      const body = await res.json();
      products.update((pr) => {
        let product = pr.get(id);
        product.ratings = body;
        pr.set(id, product);
        return pr;
      });
    }
  };
</script>

{#each [...$products.entries()] as [id, body]}
  {#if body.image}
    <img src={body.image} alt="bułka" height="300" width="300" />
  {/if}
  <h1>{body.name}</h1>
  <h2>Cena: {body.price}zł</h2>
  <h2>Twoja ocena: {body.rating || "brak"}</h2>
  {#if body.rating == undefined}
    <Rate bind:productId={id} />
  {/if}
  {#if body.ratings}
    {#if body.ratings.length > 0}
      <h3>Oceny społeczności:</h3>
      {#each body.ratings as rating}
        <p><i>{rating.username}</i>: {rating.rating}/10 🥐</p>
      {/each}
    {:else}
      <h3>Brak ocen społeczności</h3>
    {/if}
  {/if}
  <button
    on:click={async (e) => {
      e.currentTarget.disabled = true;
      loadRatings(id);
    }}>Zobacz oceny</button
  >
  <hr />
{/each}
