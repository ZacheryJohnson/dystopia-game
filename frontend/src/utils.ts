const API_GATEWAY_BASE = import.meta.env.VITE_API_GATEWAY_BASE_URI || window.location.origin;

// Executes a request against the provided API path.
// Returns the results as a parsed JSON object, which can be converted to an expected type as desired.
export const fetchApi = async (path: string, options?: RequestInit): Promise<Response> => {
    if (options) {
        if (!options.headers) {
            options.headers = new Headers({
                // 'Content-Type': 'application/json',
            });
        }
    }

    return await fetch(`${API_GATEWAY_BASE}/${path}`, options);
};
