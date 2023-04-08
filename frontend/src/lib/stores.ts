import { writable } from "svelte/store";
import type { Rating } from "./interfaces";

export const isAuthorized = writable(true); // to change

export interface InterfaceProduct {
    name: string;
    price: string;
    rating: number;
    ratings: Array<Rating>;
    image?: string;
}

export const products = writable(new Map<string, InterfaceProduct>());
