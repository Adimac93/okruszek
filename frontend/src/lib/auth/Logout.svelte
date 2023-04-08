<script lang="ts">
    import { json } from "../api";
    import { isAuthorized } from "../stores";

    const logout = async () => {
        const res = await json("/api/auth/logout", "POST");
        if (res) {
            isAuthorized.set(false);
        }
    };
</script>

<button on:click={logout}>Logout</button>
