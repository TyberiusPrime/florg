import { invoke } from "@tauri-apps/api/tauri";

export async function load({ params }: { params: any }) {
	let data = invoke("get_git_history", {
		limit: 500});
	return {
		entries: data
	}
}
 
