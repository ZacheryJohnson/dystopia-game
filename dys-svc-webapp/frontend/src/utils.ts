// Executes a request against the provided API path.
// Returns the results as a parsed JSON object, which can be converted to an expected type as desired.
export const fetchApi = async (path: string, options?: RequestInit): Promise<any> => {
    return (await fetch(`${window.location.origin}/api/${path}`, options)).json();
};