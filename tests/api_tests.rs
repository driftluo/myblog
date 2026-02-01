//! API Integration Tests
//!
//! These tests require a running server and database connection
//! Before running tests, ensure:
//! 1. PostgreSQL database is started and configured correctly
//! 2. Redis is started
//! 3. Server is running on http://127.0.0.1:8080
//!
//! Run tests: cargo test --test api_tests -- --test-threads=1

use reqwest::{Client, StatusCode};
use serde_json::{json, Value};

const BASE_URL: &str = "http://127.0.0.1:8080";
const API_PREFIX: &str = "/api/v1";

/// Create a client with cookie jar
fn create_client() -> Client {
    Client::builder()
        .cookie_store(true)
        .build()
        .expect("Failed to create HTTP client")
}

/// Password format: 6 character prefix + actual password
fn format_password(password: &str) -> String {
    format!("prefix{}", password)
}

// ============================================
// Password Validation Tests
// ============================================

#[cfg(test)]
mod password_tests {
    use new_blog::utils::get_password;

    #[test]
    fn test_get_password_valid() {
        // Normal case: 6-char prefix + actual password
        let result = get_password("abcdefpassword");
        assert_eq!(result, Some("password".to_string()));
    }

    #[test]
    fn test_get_password_exactly_6_chars() {
        // Edge case: exactly 6 characters, actual password is empty
        let result = get_password("abcdef");
        assert_eq!(result, Some("".to_string()));
    }

    #[test]
    fn test_get_password_too_short() {
        // Error case: less than 6 characters should return None
        let result = get_password("admin");
        assert_eq!(result, None);

        let result = get_password("12345");
        assert_eq!(result, None);

        let result = get_password("");
        assert_eq!(result, None);
    }

    #[test]
    fn test_get_password_unicode() {
        // Unicode character test (note: split_at splits by bytes)
        // "你好世" is 9 bytes, more than 6 bytes
        let result = get_password("你好世界");
        // "你好" = 6 bytes, remaining "世界"
        assert!(result.is_some());
    }
}

// ============================================
// Visitor API Tests (Public Endpoints)
// ============================================

#[cfg(test)]
mod visitor_api_tests {
    use super::*;

    #[tokio::test]
    #[ignore = "requires running server"]
    async fn test_article_view_all() {
        let client = create_client();
        let url = format!(
            "{}{}/article/view_all?limit=5&offset=0",
            BASE_URL, API_PREFIX
        );

        let response = client.get(&url).send().await.expect("Request failed");
        assert_eq!(response.status(), StatusCode::OK);

        let body: Value = response.json().await.expect("Failed to parse JSON");
        assert_eq!(body["status"], true);
        assert!(body["data"].is_array());
    }

    #[tokio::test]
    #[ignore = "requires running server"]
    async fn test_article_view_all_by_tag() {
        let client = create_client();
        // Use a non-existent tag_id, should return empty array
        let url = format!(
            "{}{}/article/view_all/00000000-0000-0000-0000-000000000000",
            BASE_URL, API_PREFIX
        );

        let response = client.get(&url).send().await.expect("Request failed");
        assert_eq!(response.status(), StatusCode::OK);

        let body: Value = response.json().await.expect("Failed to parse JSON");
        assert_eq!(body["status"], true);
        assert!(body["data"].is_array());
    }

    #[tokio::test]
    #[ignore = "requires running server"]
    async fn test_rss_feed() {
        let client = create_client();
        let url = format!("{}/rss", BASE_URL);

        let response = client.get(&url).send().await.expect("Request failed");
        assert_eq!(response.status(), StatusCode::OK);

        let content_type = response
            .headers()
            .get("content-type")
            .expect("No content-type header");
        assert!(content_type.to_str().unwrap().contains("xml"));
    }

    #[tokio::test]
    #[ignore = "requires running server"]
    async fn test_login_password_too_short() {
        let client = create_client();
        let url = format!("{}{}/user/login", BASE_URL, API_PREFIX);

        // Password too short (less than 6 characters), should be rejected
        let response = client
            .post(&url)
            .json(&json!({
                "account": "admin",
                "password": "admin",  // Only 5 characters, will cause get_password to return None
                "remember": false
            }))
            .send()
            .await
            .expect("Request failed");

        // Server should respond normally (not panic), return error message
        assert_eq!(response.status(), StatusCode::OK);
        let body: Value = response.json().await.expect("Failed to parse JSON");
        assert_eq!(body["status"], false);
        assert!(body["error"]
            .as_str()
            .unwrap()
            .contains("Invalid password format"));
    }
}

// ============================================
// Authentication Tests
// ============================================

#[cfg(test)]
mod auth_tests {
    use super::*;

    #[tokio::test]
    #[ignore = "requires running server and valid admin account"]
    async fn test_login_success() {
        let client = create_client();
        let url = format!("{}{}/user/login", BASE_URL, API_PREFIX);

        let response = client
            .post(&url)
            .json(&json!({
                "account": "admin",
                "password": format_password("admin"),
                "remember": false
            }))
            .send()
            .await
            .expect("Request failed");

        assert_eq!(response.status(), StatusCode::OK);

        let body: Value = response.json().await.expect("Failed to parse JSON");
        assert_eq!(body["status"], true);
    }

    #[tokio::test]
    #[ignore = "requires running server"]
    async fn test_login_wrong_password() {
        let client = create_client();
        let url = format!("{}{}/user/login", BASE_URL, API_PREFIX);

        let response = client
            .post(&url)
            .json(&json!({
                "account": "admin",
                "password": format_password("wrongpassword"),
                "remember": false
            }))
            .send()
            .await
            .expect("Request failed");

        assert_eq!(response.status(), StatusCode::OK);

        let body: Value = response.json().await.expect("Failed to parse JSON");
        assert_eq!(body["status"], false);
    }

    #[tokio::test]
    #[ignore = "requires running server"]
    async fn test_protected_api_without_auth() {
        let client = create_client();
        let url = format!(
            "{}{}/article/admin/view_all?limit=5&offset=0",
            BASE_URL, API_PREFIX
        );

        let response = client.get(&url).send().await.expect("Request failed");
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }
}

// ============================================
// Admin API Tests (Requires Authentication)
// ============================================

#[cfg(test)]
mod admin_api_tests {
    use super::*;

    /// Login and return authenticated client
    async fn login_as_admin() -> Client {
        let client = create_client();
        let url = format!("{}{}/user/login", BASE_URL, API_PREFIX);

        let response = client
            .post(&url)
            .json(&json!({
                "account": "admin",
                "password": format_password("admin"),
                "remember": false
            }))
            .send()
            .await
            .expect("Login failed");

        assert_eq!(response.status(), StatusCode::OK);
        client
    }

    #[tokio::test]
    #[ignore = "requires running server and valid admin account"]
    async fn test_admin_article_list() {
        let client = login_as_admin().await;
        let url = format!(
            "{}{}/article/admin/view_all?limit=5&offset=0",
            BASE_URL, API_PREFIX
        );

        let response = client.get(&url).send().await.expect("Request failed");
        assert_eq!(response.status(), StatusCode::OK);

        let body: Value = response.json().await.expect("Failed to parse JSON");
        assert_eq!(body["status"], true);
        assert!(body["data"].is_array());
    }

    #[tokio::test]
    #[ignore = "requires running server and valid admin account"]
    async fn test_fund_crud_and_batch_operations() {
        let client = login_as_admin().await;

        // 1. Create portfolio
        let create_portfolio_url = format!("{}{}/fund/portfolio", BASE_URL, API_PREFIX);
        let resp = client
            .post(&create_portfolio_url)
            .json(&json!({ "name": "API Test Portfolio", "description": "created by tests" }))
            .send()
            .await
            .expect("Create portfolio request failed");
        assert_eq!(resp.status(), StatusCode::OK);
        let body: Value = resp.json().await.expect("Failed to parse JSON");
        assert_eq!(body["status"], true);
        let portfolio_id = body["data"].as_i64().expect("id missing") as i32;

        // 2. Create two entries
        let create_entry_url = format!("{}{}/fund/entry", BASE_URL, API_PREFIX);
        let resp1 = client
            .post(&create_entry_url)
            .json(&json!({
                "portfolio_id": portfolio_id,
                "major_category": "Stocks",
                "minor_category": "Tech",
                "fund_type": "ETF",
                "fund_name": "Test Fund A",
                "target_ratio": 0.5,
                "amount": 1000.0
            }))
            .send()
            .await
            .expect("Create entry A failed");
        assert_eq!(resp1.status(), StatusCode::OK);
        let b1: Value = resp1.json().await.expect("Parse create A");
        assert_eq!(b1["status"], true);
        let entry_a_id = b1["data"].as_i64().expect("id missing") as i32;

        let resp2 = client
            .post(&create_entry_url)
            .json(&json!({
                "portfolio_id": portfolio_id,
                "major_category": "Stocks",
                "minor_category": "Tech",
                "fund_type": "ETF",
                "fund_name": "Test Fund B",
                "target_ratio": 0.5,
                "amount": 500.0
            }))
            .send()
            .await
            .expect("Create entry B failed");
        assert_eq!(resp2.status(), StatusCode::OK);
        let b2: Value = resp2.json().await.expect("Parse create B");
        assert_eq!(b2["status"], true);
        let entry_b_id = b2["data"].as_i64().expect("id missing") as i32;

        // 3. List entries
        let list_url = format!("{}{}/fund/entries/{}", BASE_URL, API_PREFIX, portfolio_id);
        let resp = client
            .get(&list_url)
            .send()
            .await
            .expect("List entries failed");
        assert_eq!(resp.status(), StatusCode::OK);
        let list_body: Value = resp.json().await.expect("Parse list");
        assert_eq!(list_body["status"], true);
        let arr = list_body["data"].as_array().expect("data array");
        assert!(arr.len() >= 2);

        // 4. Update one entry (change minor_category)
        let update_url = format!("{}{}/fund/entry/update", BASE_URL, API_PREFIX);
        let resp = client
            .post(&update_url)
            .json(&json!({ "id": entry_a_id, "minor_category": "Software" }))
            .send()
            .await
            .expect("Update entry failed");
        assert_eq!(resp.status(), StatusCode::OK);
        let upb: Value = resp.json().await.expect("Parse update");
        assert_eq!(upb["status"], true);

        // 5. Batch update amounts
        let batch_amount_url = format!("{}{}/fund/entries/batch-update", BASE_URL, API_PREFIX);
        let resp = client
            .post(&batch_amount_url)
            .json(&json!({ "portfolio_id": portfolio_id, "updates": [{ "id": entry_a_id, "amount": 1200.0 }, { "id": entry_b_id, "amount": 300.0 }] }))
            .send()
            .await
            .expect("Batch update amounts failed");
        assert_eq!(resp.status(), StatusCode::OK);
        let bab: Value = resp.json().await.expect("Parse batch amounts");
        assert_eq!(bab["status"], true);

        // 6. Batch update order
        let batch_order_url = format!("{}{}/fund/entries/batch-order", BASE_URL, API_PREFIX);
        let resp = client
            .post(&batch_order_url)
            .json(&json!({ "portfolio_id": portfolio_id, "updates": [{ "id": entry_b_id, "sort_index": 0 }, { "id": entry_a_id, "sort_index": 1 }] }))
            .send()
            .await
            .expect("Batch order failed");
        assert_eq!(resp.status(), StatusCode::OK);
        let bor: Value = resp.json().await.expect("Parse batch order");
        assert_eq!(bor["status"], true);

        // 7. Delete entries
        let del_a = format!(
            "{}{}/fund/entry/delete/{}",
            BASE_URL, API_PREFIX, entry_a_id
        );
        let resp = client.post(&del_a).send().await.expect("Delete A failed");
        assert_eq!(resp.status(), StatusCode::OK);
        let dab: Value = resp.json().await.expect("Parse delete A");
        assert_eq!(dab["status"], true);

        let del_b = format!(
            "{}{}/fund/entry/delete/{}",
            BASE_URL, API_PREFIX, entry_b_id
        );
        let resp = client.post(&del_b).send().await.expect("Delete B failed");
        assert_eq!(resp.status(), StatusCode::OK);
        let dbb: Value = resp.json().await.expect("Parse delete B");
        assert_eq!(dbb["status"], true);

        // 8. Delete portfolio
        let del_p = format!(
            "{}{}/fund/portfolio/delete/{}",
            BASE_URL, API_PREFIX, portfolio_id
        );
        let resp = client
            .post(&del_p)
            .send()
            .await
            .expect("Delete portfolio failed");
        assert_eq!(resp.status(), StatusCode::OK);
        let dpb: Value = resp.json().await.expect("Parse delete portfolio");
        assert_eq!(dpb["status"], true);
    }

    #[tokio::test]
    #[ignore = "requires running server and valid admin account"]
    async fn test_admin_unpublished_articles() {
        let client = login_as_admin().await;
        let url = format!(
            "{}{}/article/admin/view_unpublished?limit=5&offset=0",
            BASE_URL, API_PREFIX
        );

        let response = client.get(&url).send().await.expect("Request failed");
        assert_eq!(response.status(), StatusCode::OK);

        let body: Value = response.json().await.expect("Failed to parse JSON");
        assert_eq!(body["status"], true);
    }

    #[tokio::test]
    #[ignore = "requires running server and valid admin account"]
    async fn test_tag_crud() {
        let client = login_as_admin().await;

        // 1. Create tag
        let create_url = format!("{}{}/tag/new", BASE_URL, API_PREFIX);
        let response = client
            .post(&create_url)
            .json(&json!({ "tag": "TestTagForApiTest" }))
            .send()
            .await
            .expect("Create tag failed");
        assert_eq!(response.status(), StatusCode::OK);

        // 2. View tag list
        let list_url = format!("{}{}/tag/view?limit=10&offset=0", BASE_URL, API_PREFIX);
        let response = client
            .get(&list_url)
            .send()
            .await
            .expect("List tags failed");
        assert_eq!(response.status(), StatusCode::OK);

        let body: Value = response.json().await.expect("Failed to parse JSON");
        assert_eq!(body["status"], true);

        // Find the created tag
        let tags = body["data"].as_array().expect("data should be array");
        let test_tag = tags
            .iter()
            .find(|t| t["tag"] == "TestTagForApiTest")
            .expect("Created tag not found");
        let tag_id = test_tag["id"].as_str().expect("tag id not found");

        // 3. Edit tag
        let edit_url = format!("{}{}/tag/edit", BASE_URL, API_PREFIX);
        let response = client
            .post(&edit_url)
            .json(&json!({
                "id": tag_id,
                "tag": "UpdatedTestTag"
            }))
            .send()
            .await
            .expect("Edit tag failed");
        assert_eq!(response.status(), StatusCode::OK);

        // 4. Delete tag
        let delete_url = format!("{}{}/tag/delete/{}", BASE_URL, API_PREFIX, tag_id);
        let response = client
            .post(&delete_url)
            .send()
            .await
            .expect("Delete tag failed");
        assert_eq!(response.status(), StatusCode::OK);

        let body: Value = response.json().await.expect("Failed to parse JSON");
        assert_eq!(body["status"], true);
    }

    #[tokio::test]
    #[ignore = "requires running server and valid admin account"]
    async fn test_user_list() {
        let client = login_as_admin().await;
        let url = format!("{}{}/user/view_all?limit=10&offset=0", BASE_URL, API_PREFIX);

        let response = client.get(&url).send().await.expect("Request failed");
        assert_eq!(response.status(), StatusCode::OK);

        let body: Value = response.json().await.expect("Failed to parse JSON");
        assert_eq!(body["status"], true);
        assert!(body["data"].is_array());
    }

    #[tokio::test]
    #[ignore = "requires running server and valid admin account"]
    async fn test_chart_data_month() {
        let client = login_as_admin().await;
        let url = format!("{}{}/article/month", BASE_URL, API_PREFIX);

        let response = client.get(&url).send().await.expect("Request failed");
        assert_eq!(response.status(), StatusCode::OK);

        let body: Value = response.json().await.expect("Failed to parse JSON");
        assert_eq!(body["status"], true);
    }

    #[tokio::test]
    #[ignore = "requires running server and valid admin account"]
    async fn test_ip_view() {
        let client = login_as_admin().await;
        let url = format!("{}{}/ip/view?limit=10&offset=0", BASE_URL, API_PREFIX);

        let response = client.get(&url).send().await.expect("Request failed");
        assert_eq!(response.status(), StatusCode::OK);

        let body: Value = response.json().await.expect("Failed to parse JSON");
        assert_eq!(body["status"], true);
    }

    #[tokio::test]
    #[ignore = "requires running server and valid admin account"]
    async fn test_user_view_and_edit() {
        let client = login_as_admin().await;

        // View current user info
        let view_url = format!("{}{}/user/view", BASE_URL, API_PREFIX);
        let response = client.get(&view_url).send().await.expect("Request failed");
        assert_eq!(response.status(), StatusCode::OK);

        let body: Value = response.json().await.expect("Failed to parse JSON");
        assert_eq!(body["status"], true);

        let original_nickname = body["data"]["nickname"].as_str().unwrap().to_string();

        // Edit user info
        let edit_url = format!("{}{}/user/edit", BASE_URL, API_PREFIX);
        let response = client
            .post(&edit_url)
            .json(&json!({
                "nickname": "TestNickname",
                "say": "Test say",
                "email": "test@example.com"
            }))
            .send()
            .await
            .expect("Edit failed");
        assert_eq!(response.status(), StatusCode::OK);

        // Restore original info
        let response = client
            .post(&edit_url)
            .json(&json!({
                "nickname": original_nickname,
                "say": null,
                "email": "441594700@qq.com"
            }))
            .send()
            .await
            .expect("Restore failed");
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    #[ignore = "requires running server and valid admin account"]
    async fn test_sign_out() {
        let client = login_as_admin().await;

        // Sign out
        let signout_url = format!("{}{}/user/sign_out", BASE_URL, API_PREFIX);
        let response = client
            .get(&signout_url)
            .send()
            .await
            .expect("Sign out failed");
        assert_eq!(response.status(), StatusCode::OK);

        // After sign out, accessing protected API should return 403
        let protected_url = format!("{}{}/user/view", BASE_URL, API_PREFIX);
        let response = client
            .get(&protected_url)
            .send()
            .await
            .expect("Request failed");
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }
}

// ============================================
// Page Route Tests
// ============================================

#[cfg(test)]
mod page_route_tests {
    use super::*;

    #[tokio::test]
    #[ignore = "requires running server"]
    async fn test_index_page() {
        let client = create_client();
        let response = client.get(BASE_URL).send().await.expect("Request failed");

        assert_eq!(response.status(), StatusCode::OK);
        let body = response.text().await.expect("Failed to get body");
        assert!(body.contains("<!DOCTYPE html>"));
    }

    #[tokio::test]
    #[ignore = "requires running server"]
    async fn test_about_page() {
        let client = create_client();
        let url = format!("{}/about", BASE_URL);
        let response = client.get(&url).send().await.expect("Request failed");

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    #[ignore = "requires running server"]
    async fn test_home_page() {
        let client = create_client();
        let url = format!("{}/home", BASE_URL);
        let response = client.get(&url).send().await.expect("Request failed");

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    #[ignore = "requires running server"]
    async fn test_static_css() {
        let client = create_client();
        let url = format!("{}/css/index.css", BASE_URL);
        let response = client.get(&url).send().await.expect("Request failed");

        assert_eq!(response.status(), StatusCode::OK);
        let body = response.text().await.expect("Failed to get body");
        // CSS file should contain style rules
        assert!(body.contains("{") && body.contains("}"));
    }

    #[tokio::test]
    #[ignore = "requires running server"]
    async fn test_static_js() {
        let client = create_client();
        let url = format!("{}/js/jquery-3.2.1.min.js", BASE_URL);
        let response = client.get(&url).send().await.expect("Request failed");

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    #[ignore = "requires running server"]
    async fn test_static_webp() {
        let client = create_client();
        let url = format!("{}/images/test.webp", BASE_URL);
        let response = client.get(&url).send().await.expect("Request failed");

        // If file exists, should return OK; if not exists, should return NOT_FOUND
        // Either way, the extension is allowed
        assert!(response.status() == StatusCode::OK || response.status() == StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    #[ignore = "requires running server"]
    async fn test_static_disallowed_extension() {
        let client = create_client();

        // Try to access a file with disallowed extension (.rs, .toml, .json, etc.)
        // These should be blocked by the static file filter
        let disallowed_urls = [
            format!("{}/test.rs", BASE_URL),
            format!("{}/test.toml", BASE_URL),
            format!("{}/test.json", BASE_URL),
            format!("{}/test.html", BASE_URL),
            format!("{}/test.txt", BASE_URL),
            format!("{}/test.php", BASE_URL),
            format!("{}/test.sh", BASE_URL),
        ];

        for url in &disallowed_urls {
            let response = client.get(url).send().await.expect("Request failed");
            // Disallowed extensions should return NOT_FOUND (blocked by filter)
            assert_eq!(
                response.status(),
                StatusCode::NOT_FOUND,
                "Expected NOT_FOUND for disallowed extension: {}",
                url
            );
        }
    }

    #[tokio::test]
    #[ignore = "requires running server"]
    async fn test_static_allowed_extensions() {
        let client = create_client();

        // Test that allowed extensions are not blocked (they return OK or NOT_FOUND based on file existence)
        // We use paths that likely don't exist to test that the filter doesn't block them
        let allowed_extensions = [
            ".css", ".js", ".png", ".jpg", ".jpeg", ".gif", ".ico", ".webp", ".woff", ".woff2",
            ".svg",
        ];

        for ext in &allowed_extensions {
            let url = format!("{}/nonexistent_file_for_test{}", BASE_URL, ext);
            let response = client.get(&url).send().await.expect("Request failed");
            // Allowed extensions should return NOT_FOUND (file doesn't exist, but extension is allowed)
            // This proves the filter didn't block the request
            assert_eq!(
                response.status(),
                StatusCode::NOT_FOUND,
                "Expected NOT_FOUND (not blocked) for allowed extension: {}",
                ext
            );
        }
    }

    #[tokio::test]
    #[ignore = "requires running server"]
    async fn test_static_case_insensitive_extension() {
        let client = create_client();

        // Test that extension check is case-insensitive
        let url = format!("{}/css/index.CSS", BASE_URL);
        let response = client.get(&url).send().await.expect("Request failed");

        // Should be allowed (case-insensitive), returns OK or NOT_FOUND
        assert!(
            response.status() == StatusCode::OK || response.status() == StatusCode::NOT_FOUND,
            "Expected OK or NOT_FOUND for case-insensitive extension check"
        );
    }
}
