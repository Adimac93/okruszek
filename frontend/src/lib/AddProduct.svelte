<script lang="ts">
  import { Fetch } from "./api";

  let name;
  let price;
  let image = null;

  let prompt = false;

  const api = new Fetch();
  const createProduct = async () => {
    const res = api.json("/api/products", "PUT", { name, price, image });
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
    let image = e.target.files[0];
    let reader = new FileReader();
    reader.readAsDataURL(image);
    reader.onload = (e) => {
      alert("Image loaded");
      image = e.target.result;
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
