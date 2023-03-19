<script lang="ts">
  import Login from "./Login.svelte";
  import Register from "./Register.svelte";

  let isLogin = true;
</script>

<p>Witaj!</p>
{#if isLogin}
  <Login />
  <button on:click={() => (isLogin = !isLogin)}>Nie masz konta?</button>
{:else}
  <Register />
  <button on:click={() => (isLogin = !isLogin)}>Masz konto?</button>
{/if}

<br />
