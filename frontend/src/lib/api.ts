import { isAuthorized } from "./stores";

type Method = "GET" | "POST" | "PUT" | "PATCH" | "DELETE";

export class Fetch {
  async json(uri: string, method: Method, body?: any) {
    let res;
    if (body == undefined) {
      res = await fetch(uri, {
        method,
      });
    }
    res = await fetch(uri, {
      body: JSON.stringify(body),
      headers: { "Content-Type": "application/json" },
      method,
    });
    return await this.handleResponse(res);
  }

  private async handleResponse(res: Response) {
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
}
