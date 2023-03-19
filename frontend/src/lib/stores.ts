import { writable } from "svelte/store";

export const isAuthorized = writable(true); // to change

interface Rating {
  username: string;
  rating: number;
}

export interface Product {
  name: string;
  price: string;
  rating: number;
  ratings: Array<Rating>;
  image?: string;
}

export const products = writable(new Map<string, Product>());
