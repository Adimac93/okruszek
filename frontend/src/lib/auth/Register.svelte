<script lang="ts">
    import { json } from "../api";
    import type { RegisterCredentials } from "../interfaces";
    import { isAuthorized } from "../stores";

    let email = "";
    let username = "";
    let passA = "";
    let passB = "";

    const register = async () => {
        if (passA != passB || passA == "") {
            alert("Passwords are not the same");
            return;
        }
        const res = await json<RegisterCredentials>(
            "/api/auth/register",
            "POST",
            {
                email,
                username,
                password: passA,
            }
        );

        if (res) {
            isAuthorized.set(true);
            console.debug("Registered");
        }
    };
</script>

<input type="email" placeholder="email" bind:value={email} />
<input type="text" placeholder="username" bind:value={username} />
<input type="password" placeholder="password" bind:value={passA} />
<input type="password" placeholder="password" bind:value={passB} />
<button on:click={register}>Register</button>

<style>
    input {
        display: block;
        margin: auto;
        margin-bottom: 0.5em;
        align-items: center;
        text-align: center;
    }
</style>
