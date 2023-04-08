<script lang="ts">
    import { isAuthorized } from "./lib/stores";
    import Dashboard from "./lib/Dashboard.svelte";
    import Auth from "./lib/auth/Auth.svelte";

    isAuthorized.subscribe((value) => {
        alert(`Authorized: ${value}`);
    });
</script>

<main>
    {#if $isAuthorized}
        <Dashboard />
    {:else}
        <Auth />
    {/if}
</main>
