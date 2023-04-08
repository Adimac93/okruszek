<script lang="ts">
    import { Fetch } from "../api";
    import { isAuthorized } from "../stores";
    import type { LoginCredentials } from "../interfaces";

    let email = "";
    let password = "";

    const api = new Fetch();

    const login = async () => {
        const res = await api.json<LoginCredentials>(
            "/api/auth/login",
            "POST",
            { email, password }
        );
        if (res) {
            isAuthorized.set(true);
            console.debug("Logged in");
        }
    };
</script>

<input type="email" placeholder="email" bind:value={email} />
<input type="password" placeholder="password" bind:value={password} />
<button on:click={login}>Login</button>

<style>
    input {
        display: block;
        margin: auto;
        margin-bottom: 0.5em;
        align-items: center;
        text-align: center;
    }
</style>
