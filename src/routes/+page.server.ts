/** @type {import('./$types').PageServerLoad} */
import { redirect } from "@sveltejs/kit";
export async function load({ params }) {
	console.log(params);
	    throw redirect(307, '/tree/');
}
