<script lang="ts">
    import { json } from "../api";
    import type { AddProduct } from "../interfaces";

    let name;
    let price;
    let image = null;

    let prompt = false;

    const createProduct = async () => {
        const res = json<AddProduct>("/api/products", "PUT", {
            name,
            price,
            image,
        });
        if (res) {
            name = undefined;
            price = undefined;
            prompt = true;
            setTimeout(() => {
                prompt = false;
            }, 3000);
        }
    };

    let fileinput;
    const onFileSelected = (e) => {
        const file = e.target.files[0];
        let reader = new FileReader();
        reader.readAsDataURL(file);
        reader.onload = (e) => {
            alert("Image loaded");
            image = (e.target.result as string).split(",")[1];
        };
    };
</script>

<input type="text" placeholder="name" bind:value={name} />
<input type="number" placeholder="price" bind:value={price} />

<input
    type="file"
    placeholder="image"
    accept=".jpg, .jpeg, .png"
    on:change={(e) => onFileSelected(e)}
    bind:this={fileinput}
/>

<button
    disabled={name == undefined || price == undefined}
    on:click={createProduct}>Dodaj produkt</button
>
{#if prompt}
    <i>Pomyślnie dodano produkt</i>
{/if}

<style>
    input {
        display: block;
        margin: auto;
        margin-bottom: 0.5em;
        align-items: center;
        text-align: center;
    }
</style>
