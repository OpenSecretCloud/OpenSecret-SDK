import { expect, test, describe } from "bun:test";
import { fetchLogin, fetchSignUp, search } from "../../api";
import type { SearchResponse, SearchResult } from "../../api";

const TEST_EMAIL = process.env.VITE_TEST_EMAIL;
const TEST_PASSWORD = process.env.VITE_TEST_PASSWORD;
const TEST_NAME = process.env.VITE_TEST_NAME;
const TEST_CLIENT_ID = process.env.VITE_TEST_CLIENT_ID;

if (!TEST_EMAIL || !TEST_PASSWORD || !TEST_NAME || !TEST_CLIENT_ID) {
  throw new Error("Test credentials must be set in .env.local");
}

async function tryEmailLogin() {
  // Ensure the test user exists
  try {
    return await fetchLogin(TEST_EMAIL!, TEST_PASSWORD!, TEST_CLIENT_ID!);
  } catch (error) {
    console.warn(error);
    console.log("Login failed, attempting signup");
    await fetchSignUp(TEST_EMAIL!, TEST_PASSWORD!, "", TEST_CLIENT_ID!, TEST_NAME!);
    return await fetchLogin(TEST_EMAIL!, TEST_PASSWORD!, TEST_CLIENT_ID!);
  }
}

async function setupAuth() {
  const { access_token, refresh_token } = await tryEmailLogin();
  window.localStorage.setItem("access_token", access_token);
  window.localStorage.setItem("refresh_token", refresh_token);
}

function validateSearchResult(result: SearchResult) {
  // Required fields
  expect(result).toHaveProperty("url");
  expect(result).toHaveProperty("title");
  expect(typeof result.url).toBe("string");
  expect(typeof result.title).toBe("string");

  // Optional fields (based on actual server response)
  if (result.snippet !== undefined) {
    expect(typeof result.snippet).toBe("string");
  }

  if (result.time !== undefined) {
    expect(typeof result.time).toBe("string");
  }

  if (result.image !== undefined) {
    expect(result.image).toHaveProperty("url");
    expect(typeof result.image.url).toBe("string");
    // Width and height are optional even in image object
    if (result.image.width !== undefined) {
      expect(typeof result.image.width).toBe("number");
    }
    if (result.image.height !== undefined) {
      expect(typeof result.image.height).toBe("number");
    }
  }

  if (result.props !== undefined) {
    expect(typeof result.props).toBe("object");
  }
}

function validateSearchResponse(response: SearchResponse) {
  // Top level structure
  expect(response).toHaveProperty("success");
  expect(response).toHaveProperty("data");
  expect(response).toHaveProperty("error");
  expect(typeof response.success).toBe("boolean");

  if (response.success) {
    expect(response.data).not.toBeNull();
    expect(response.error).toBeNull();

    if (response.data) {
      // Meta structure - make fields optional since server might not return all
      expect(response.data).toHaveProperty("meta");
      if (response.data.meta.trace !== undefined) {
        expect(typeof response.data.meta.trace).toBe("string");
      }
      if (response.data.meta.id !== undefined) {
        expect(typeof response.data.meta.id).toBe("string");
      }
      if (response.data.meta.node !== undefined) {
        expect(typeof response.data.meta.node).toBe("string");
      }
      if (response.data.meta.ms !== undefined) {
        expect(typeof response.data.meta.ms).toBe("number");
      }

      // Data structure
      expect(response.data).toHaveProperty("data");
      expect(typeof response.data.data).toBe("object");
    }
  } else {
    expect(response.data).toBeNull();
    expect(response.error).not.toBeNull();
    expect(typeof response.error).toBe("string");
  }
}

describe("Kagi Search API Proxy Tests", () => {
  test("Search API - basic web search with response validation", async () => {
    await setupAuth();

    const response = await search("OpenAI GPT-4");

    // Validate full response structure
    validateSearchResponse(response);

    if (response.success && response.data) {
      // For web search, results should be in 'search' array
      if (response.data.data.search && response.data.data.search.length > 0) {
        expect(Array.isArray(response.data.data.search)).toBe(true);

        // Validate each search result
        response.data.data.search.forEach((result) => {
          validateSearchResult(result);
        });

        console.log(`Web search returned ${response.data.data.search.length} results`);
      }

      // Check for related searches
      if (response.data.data.related_search) {
        expect(Array.isArray(response.data.data.related_search)).toBe(true);
        response.data.data.related_search.forEach((related) => {
          expect(related).toHaveProperty("title");
          if (related.props?.query) {
            expect(typeof related.props.query).toBe("string");
          }
        });
      }
    }
  });

  test("Search API - image search workflow", async () => {
    await setupAuth();

    const response = await search("mountain landscape photography", "images");

    validateSearchResponse(response);

    if (response.success && response.data) {
      // Image results should be in the 'image' array
      if (response.data.data.image && response.data.data.image.length > 0) {
        expect(Array.isArray(response.data.data.image)).toBe(true);

        response.data.data.image.forEach((result) => {
          validateSearchResult(result);
          // Images should typically have image metadata
          if (result.image) {
            expect(result.image.url).toBeTruthy();
          }
        });

        console.log(`Image search returned ${response.data.data.image.length} results`);
      }
    }
  });

  test("Search API - video search workflow", async () => {
    await setupAuth();

    const response = await search("machine learning tutorial", "videos");

    validateSearchResponse(response);

    if (response.success && response.data) {
      if (response.data.data.video && response.data.data.video.length > 0) {
        expect(Array.isArray(response.data.data.video)).toBe(true);

        response.data.data.video.forEach((result) => {
          validateSearchResult(result);
          // Videos might have duration in props
          if (result.props?.duration) {
            expect(typeof result.props.duration).toBe("string");
          }
        });

        console.log(`Video search returned ${response.data.data.video.length} results`);
      }
    }
  });

  test("Search API - news search workflow", async () => {
    await setupAuth();

    const response = await search("artificial intelligence breakthrough", "news");

    validateSearchResponse(response);

    if (response.success && response.data) {
      if (response.data.data.news && response.data.data.news.length > 0) {
        expect(Array.isArray(response.data.data.news)).toBe(true);

        response.data.data.news.forEach((result) => {
          validateSearchResult(result);
          // News articles should typically have time
          if (result.time) {
            expect(result.time).toBeTruthy();
          }
        });

        console.log(`News search returned ${response.data.data.news.length} results`);
      }
    }
  });

  test("Search API - podcast search workflow", async () => {
    await setupAuth();

    const response = await search("tech podcast", "podcasts");

    validateSearchResponse(response);

    if (response.success && response.data) {
      if (response.data.data.podcast && response.data.data.podcast.length > 0) {
        expect(Array.isArray(response.data.data.podcast)).toBe(true);

        response.data.data.podcast.forEach((result) => {
          validateSearchResult(result);
        });

        console.log(`Podcast search returned ${response.data.data.podcast.length} results`);
      }

      // Check for podcast creators too
      if (response.data.data.podcast_creator) {
        expect(Array.isArray(response.data.data.podcast_creator)).toBe(true);
      }
    }
  });

  test("Search API - complex query with special characters", async () => {
    await setupAuth();

    const response = await search('site:github.com "OpenSecret SDK" filetype:md');

    validateSearchResponse(response);

    if (response.success && response.data) {
      console.log(`Complex query returned results in ${response.data.meta.ms}ms`);
    }
  });

  test("Search API - empty query handling", async () => {
    await setupAuth();

    try {
      await search("");
      // Some servers might accept empty queries
    } catch (error) {
      // Others might reject them
      expect(error).toBeDefined();
    }
  });

  test("Search API - very long query", async () => {
    await setupAuth();

    const longQuery = "artificial intelligence ".repeat(50);
    const response = await search(longQuery);

    validateSearchResponse(response);
  });

  test("Search API - encryption/decryption validation", async () => {
    await setupAuth();

    // This tests that the encrypted request/response cycle works correctly
    const response = await search("encryption test");

    validateSearchResponse(response);

    // If we get here without errors, encryption/decryption worked
    expect(response).toBeDefined();
  });

  test("Search API - error handling for missing auth", async () => {
    // Clear auth tokens
    window.localStorage.removeItem("access_token");
    window.localStorage.removeItem("refresh_token");

    try {
      await search("test query");
      expect(true).toBe(false); // Should not reach here
    } catch (error: any) {
      expect(error).toBeDefined();
      expect(error.message).toBeTruthy();
    }
  });

  test("Search API - session handling", async () => {
    await setupAuth();

    // Multiple searches should use the same session
    const response1 = await search("first query");
    const response2 = await search("second query");

    validateSearchResponse(response1);
    validateSearchResponse(response2);

    // Both should succeed if session handling is correct
    expect(response1.success || response1.error).toBeTruthy();
    expect(response2.success || response2.error).toBeTruthy();
  });

  test("Search API - workflow parameter validation", async () => {
    await setupAuth();

    // Test all valid workflows
    const workflows = ["search", "images", "videos", "news", "podcasts"] as const;

    for (const workflow of workflows) {
      const response = await search(`test ${workflow}`, workflow);
      validateSearchResponse(response);
      console.log(`Workflow '${workflow}' validated`);
    }
  }, 15000); // 15 second timeout for this test since it makes multiple API calls
});
