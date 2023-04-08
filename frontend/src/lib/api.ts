import { isAuthorized } from "./stores";


type Method = "GET" | "POST" | "PUT" | "PATCH" | "DELETE";


export async function json<B = any>(uri: string, method: Method, body?: B) {
    console.log(`Fetching ${uri}`)
    let res;
    if (body == undefined) {
        res = await fetch(uri, { method });
        return await handleResponse(res);
    }


    res = await fetch(uri, {
        body: JSON.stringify(body),
        headers: { "Content-Type": "application/json" },
        method,
    });
    return await handleResponse(res);
}

async function handleResponse(res: Response) {
    if (res.ok) return res;

    if (res.status == 401) {
        isAuthorized.set(false);
    }

    const body = await res.json();
    const keys = Object.keys(body);
    if (keys.length > 0 && keys.includes("errorInfo")) {
        alert(body.errorInfo);
    } else {
        alert("Unknown error, report to Adam");
    }
    return false;
}
