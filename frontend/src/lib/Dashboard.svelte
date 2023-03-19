<script lang="ts">
  import AddProduct from "./AddProduct.svelte";
  import Logout from "./auth/Logout.svelte";
  import Load from "./files/Load.svelte";
  import Upload from "./files/Upload.svelte";
  import Products from "./Products.svelte";
</script>

<Logout />
<div>
  <br />
  <AddProduct />
</div>

<Products />
<br />
