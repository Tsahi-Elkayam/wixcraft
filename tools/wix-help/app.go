package main

import (
	"context"
	"database/sql"
	"fmt"
	"os"
	"path/filepath"
	"strings"

	_ "github.com/mattn/go-sqlite3"
)

// App struct
type App struct {
	ctx context.Context
	db  *sql.DB
}

// NewApp creates a new App application struct
func NewApp() *App {
	return &App{}
}

// startup is called when the app starts
func (a *App) startup(ctx context.Context) {
	a.ctx = ctx

	// Find database path
	exePath, err := os.Executable()
	if err != nil {
		fmt.Println("Error getting executable path:", err)
		return
	}

	// Try various paths to find the database
	homeDir, _ := os.UserHomeDir()
	dbPaths := []string{
		filepath.Join(filepath.Dir(exePath), "..", "data", "wix.db"),
		filepath.Join(filepath.Dir(exePath), "data", "wix.db"),
		filepath.Join(homeDir, ".wixcraft", "wix.db"),
		"../common/wix-data/wix.db",
		"../../common/wix-data/wix.db",
		"common/wix-data/wix.db",
		"data/wix.db",
	}

	var dbPath string
	for _, p := range dbPaths {
		if _, err := os.Stat(p); err == nil {
			dbPath = p
			break
		}
	}

	if dbPath == "" {
		fmt.Println("Error: Could not find wix.db")
		return
	}

	fmt.Println("Using database:", dbPath)

	db, err := sql.Open("sqlite3", dbPath+"?mode=ro")
	if err != nil {
		fmt.Println("Error opening database:", err)
		return
	}
	a.db = db
}

// shutdown is called when the app is closing
func (a *App) shutdown(ctx context.Context) {
	if a.db != nil {
		a.db.Close()
	}
}

// TreeNode represents a node in the sidebar tree
type TreeNode struct {
	ID       string     `json:"id"`
	Name     string     `json:"name"`
	Type     string     `json:"type"`
	Icon     string     `json:"icon"`
	Children []TreeNode `json:"children,omitempty"`
}

// ElementInfo represents a WiX element
type ElementInfo struct {
	ID            int64           `json:"id"`
	Name          string          `json:"name"`
	Namespace     string          `json:"namespace"`
	SinceVersion  string          `json:"since_version"`
	Description   string          `json:"description"`
	Documentation string          `json:"documentation"`
	Remarks       string          `json:"remarks"`
	Parents       []string        `json:"parents"`
	Children      []string        `json:"children"`
	Attributes    []AttributeInfo `json:"attributes"`
}

// AttributeInfo represents an element attribute
type AttributeInfo struct {
	Name         string   `json:"name"`
	Type         string   `json:"type"`
	Required     bool     `json:"required"`
	DefaultValue string   `json:"default_value"`
	Description  string   `json:"description"`
	EnumValues   []string `json:"enum_values"`
}

// SnippetInfo represents a code snippet
type SnippetInfo struct {
	ID          int64  `json:"id"`
	Name        string `json:"name"`
	Prefix      string `json:"prefix"`
	Description string `json:"description"`
	Body        string `json:"body"`
	Scope       string `json:"scope"`
}

// ErrorInfo represents a WiX error
type ErrorInfo struct {
	ID          int64  `json:"id"`
	Code        string `json:"code"`
	Severity    string `json:"severity"`
	Message     string `json:"message"`
	Description string `json:"description"`
	Resolution  string `json:"resolution"`
}

// IceRuleInfo represents an ICE rule
type IceRuleInfo struct {
	ID          int64  `json:"id"`
	Code        string `json:"code"`
	Severity    string `json:"severity"`
	Description string `json:"description"`
	Tables      string `json:"tables"`
	Resolution  string `json:"resolution"`
}

// RuleInfo represents a lint rule
type RuleInfo struct {
	ID            int64  `json:"id"`
	RuleID        string `json:"rule_id"`
	Category      string `json:"category"`
	Severity      string `json:"severity"`
	Name          string `json:"name"`
	Description   string `json:"description"`
	Rationale     string `json:"rationale"`
	FixSuggestion string `json:"fix_suggestion"`
	TargetName    string `json:"target_name"`
}

// SearchResult represents a search result
type SearchResult struct {
	ID       string `json:"id"`
	Name     string `json:"name"`
	Type     string `json:"type"`
	Category string `json:"category"`
	Synopsis string `json:"synopsis"`
}

// GetSidebarTree returns the sidebar tree structure
func (a *App) GetSidebarTree() []TreeNode {
	tree := []TreeNode{}

	// 1. Elements by namespace
	elementsNode := TreeNode{ID: "elements", Name: "Elements", Type: "category", Icon: "cube", Children: []TreeNode{}}
	namespaces := a.getNamespaces()
	for _, ns := range namespaces {
		nsNode := TreeNode{ID: "ns:" + ns, Name: ns, Type: "namespace", Children: []TreeNode{}}
		elements := a.getElementsByNamespace(ns)
		for _, e := range elements {
			nsNode.Children = append(nsNode.Children, TreeNode{ID: "element:" + e, Name: e, Type: "element"})
		}
		elementsNode.Children = append(elementsNode.Children, nsNode)
	}
	tree = append(tree, elementsNode)

	// 2. Examples (code snippets)
	examplesNode := TreeNode{ID: "examples", Name: "Examples", Type: "category", Icon: "code", Children: []TreeNode{}}
	snippets := a.getSnippetNames()
	for _, s := range snippets {
		examplesNode.Children = append(examplesNode.Children, TreeNode{ID: "snippet:" + s, Name: s, Type: "snippet"})
	}
	tree = append(tree, examplesNode)

	// 3. Errors
	errorsNode := TreeNode{ID: "errors", Name: "Errors", Type: "category", Icon: "warning", Children: []TreeNode{}}

	// WiX Errors
	wixErrorsNode := TreeNode{ID: "wix-errors", Name: "WiX Errors", Type: "error-type", Children: []TreeNode{}}
	wixErrors := a.getWixErrorCodes()
	for _, e := range wixErrors {
		wixErrorsNode.Children = append(wixErrorsNode.Children, TreeNode{ID: "error:" + e, Name: e, Type: "wix-error"})
	}
	errorsNode.Children = append(errorsNode.Children, wixErrorsNode)

	// ICE Rules
	iceNode := TreeNode{ID: "ice-rules", Name: "ICE Rules", Type: "error-type", Children: []TreeNode{}}
	iceRules := a.getIceRuleCodes()
	for _, r := range iceRules {
		iceNode.Children = append(iceNode.Children, TreeNode{ID: "ice:" + r, Name: r, Type: "ice-rule"})
	}
	errorsNode.Children = append(errorsNode.Children, iceNode)

	tree = append(tree, errorsNode)

	// 4. Best Practices / Guidelines
	rulesNode := TreeNode{ID: "guidelines", Name: "Best Practices", Type: "category", Icon: "check", Children: []TreeNode{}}
	categories := a.getRuleCategories()
	for _, cat := range categories {
		catNode := TreeNode{ID: "rule-cat:" + cat, Name: cat, Type: "rule-category", Children: []TreeNode{}}
		rules := a.getRulesByCategory(cat)
		for _, r := range rules {
			catNode.Children = append(catNode.Children, TreeNode{ID: "rule:" + r, Name: r, Type: "rule"})
		}
		rulesNode.Children = append(rulesNode.Children, catNode)
	}
	tree = append(tree, rulesNode)

	return tree
}

func (a *App) getNamespaces() []string {
	var namespaces []string
	rows, err := a.db.Query(`SELECT DISTINCT namespace FROM elements WHERE namespace != '' ORDER BY namespace`)
	if err != nil {
		return namespaces
	}
	defer rows.Close()

	for rows.Next() {
		var ns string
		rows.Scan(&ns)
		namespaces = append(namespaces, ns)
	}
	return namespaces
}

func (a *App) getElementsByNamespace(namespace string) []string {
	var elements []string
	rows, err := a.db.Query(`SELECT name FROM elements WHERE namespace = ? ORDER BY name`, namespace)
	if err != nil {
		return elements
	}
	defer rows.Close()

	for rows.Next() {
		var n string
		rows.Scan(&n)
		elements = append(elements, n)
	}
	return elements
}

func (a *App) getSnippetNames() []string {
	var snippets []string
	rows, err := a.db.Query(`SELECT name FROM snippets ORDER BY name`)
	if err != nil {
		return snippets
	}
	defer rows.Close()

	for rows.Next() {
		var n string
		rows.Scan(&n)
		snippets = append(snippets, n)
	}
	return snippets
}

func (a *App) getWixErrorCodes() []string {
	var codes []string
	rows, err := a.db.Query(`SELECT code FROM errors ORDER BY code`)
	if err != nil {
		return codes
	}
	defer rows.Close()

	for rows.Next() {
		var c string
		rows.Scan(&c)
		codes = append(codes, c)
	}
	return codes
}

func (a *App) getIceRuleCodes() []string {
	var codes []string
	rows, err := a.db.Query(`SELECT code FROM ice_rules ORDER BY code`)
	if err != nil {
		return codes
	}
	defer rows.Close()

	for rows.Next() {
		var c string
		rows.Scan(&c)
		codes = append(codes, c)
	}
	return codes
}

func (a *App) getRuleCategories() []string {
	var categories []string
	rows, err := a.db.Query(`SELECT DISTINCT category FROM rules WHERE category != '' ORDER BY category`)
	if err != nil {
		return categories
	}
	defer rows.Close()

	for rows.Next() {
		var c string
		rows.Scan(&c)
		categories = append(categories, c)
	}
	return categories
}

func (a *App) getRulesByCategory(category string) []string {
	var rules []string
	rows, err := a.db.Query(`SELECT rule_id FROM rules WHERE category = ? ORDER BY rule_id`, category)
	if err != nil {
		return rules
	}
	defer rows.Close()

	for rows.Next() {
		var r string
		rows.Scan(&r)
		rules = append(rules, r)
	}
	return rules
}

// GetElement returns full element info by name
func (a *App) GetElement(name string) *ElementInfo {
	var e ElementInfo
	err := a.db.QueryRow(`
		SELECT id, name, COALESCE(namespace, ''), COALESCE(since_version, ''),
			COALESCE(description, ''), COALESCE(documentation_url, ''), COALESCE(remarks, '')
		FROM elements WHERE name = ?
	`, name).Scan(&e.ID, &e.Name, &e.Namespace, &e.SinceVersion, &e.Description, &e.Documentation, &e.Remarks)
	if err != nil {
		return nil
	}

	// Get parents
	e.Parents = a.getElementParents(e.ID)

	// Get children
	e.Children = a.getElementChildren(e.ID)

	// Get attributes
	e.Attributes = a.getElementAttributes(e.ID)

	return &e
}

func (a *App) getElementParents(elementID int64) []string {
	var parents []string
	rows, err := a.db.Query(`
		SELECT e.name FROM elements e
		JOIN element_parents ep ON e.id = ep.parent_id
		WHERE ep.element_id = ?
		ORDER BY e.name
	`, elementID)
	if err != nil {
		return parents
	}
	defer rows.Close()

	for rows.Next() {
		var p string
		rows.Scan(&p)
		parents = append(parents, p)
	}
	return parents
}

func (a *App) getElementChildren(elementID int64) []string {
	var children []string
	rows, err := a.db.Query(`
		SELECT e.name FROM elements e
		JOIN element_children ec ON e.id = ec.child_id
		WHERE ec.element_id = ?
		ORDER BY e.name
	`, elementID)
	if err != nil {
		return children
	}
	defer rows.Close()

	for rows.Next() {
		var c string
		rows.Scan(&c)
		children = append(children, c)
	}
	return children
}

func (a *App) getElementAttributes(elementID int64) []AttributeInfo {
	var attrs []AttributeInfo
	rows, err := a.db.Query(`
		SELECT name, COALESCE(attr_type, 'string'), COALESCE(required, 0),
			COALESCE(default_value, ''), COALESCE(description, '')
		FROM attributes WHERE element_id = ?
		ORDER BY required DESC, name
	`, elementID)
	if err != nil {
		return attrs
	}
	defer rows.Close()

	for rows.Next() {
		var a AttributeInfo
		var required int
		rows.Scan(&a.Name, &a.Type, &required, &a.DefaultValue, &a.Description)
		a.Required = required == 1
		attrs = append(attrs, a)
	}
	return attrs
}

// GetSnippet returns snippet by name
func (a *App) GetSnippet(name string) *SnippetInfo {
	var s SnippetInfo
	err := a.db.QueryRow(`
		SELECT id, name, COALESCE(prefix, ''), COALESCE(description, ''),
			COALESCE(body, ''), COALESCE(scope, '')
		FROM snippets WHERE name = ?
	`, name).Scan(&s.ID, &s.Name, &s.Prefix, &s.Description, &s.Body, &s.Scope)
	if err != nil {
		return nil
	}
	return &s
}

// GetWixError returns WiX error by code
func (a *App) GetWixError(code string) *ErrorInfo {
	var e ErrorInfo
	err := a.db.QueryRow(`
		SELECT id, code, COALESCE(severity, ''), COALESCE(message_template, ''),
			COALESCE(description, ''), COALESCE(resolution, '')
		FROM errors WHERE code = ?
	`, code).Scan(&e.ID, &e.Code, &e.Severity, &e.Message, &e.Description, &e.Resolution)
	if err != nil {
		return nil
	}
	return &e
}

// GetIceRule returns ICE rule by code
func (a *App) GetIceRule(code string) *IceRuleInfo {
	var r IceRuleInfo
	err := a.db.QueryRow(`
		SELECT id, code, COALESCE(severity, ''), COALESCE(description, ''),
			COALESCE(tables_affected, ''), COALESCE(resolution, '')
		FROM ice_rules WHERE code = ?
	`, code).Scan(&r.ID, &r.Code, &r.Severity, &r.Description, &r.Tables, &r.Resolution)
	if err != nil {
		return nil
	}
	return &r
}

// GetRule returns lint rule by ID
func (a *App) GetRule(ruleID string) *RuleInfo {
	var r RuleInfo
	err := a.db.QueryRow(`
		SELECT id, rule_id, COALESCE(category, ''), COALESCE(severity, ''),
			COALESCE(name, ''), COALESCE(description, ''),
			COALESCE(rationale, ''), COALESCE(fix_suggestion, ''),
			COALESCE(target_name, '')
		FROM rules WHERE rule_id = ?
	`, ruleID).Scan(&r.ID, &r.RuleID, &r.Category, &r.Severity, &r.Name,
		&r.Description, &r.Rationale, &r.FixSuggestion, &r.TargetName)
	if err != nil {
		return nil
	}
	return &r
}

// Search searches across all content
func (a *App) Search(query string) []SearchResult {
	var results []SearchResult

	if query == "" {
		return results
	}

	likeQuery := "%" + strings.ToLower(query) + "%"

	// Search elements
	rows, _ := a.db.Query(`
		SELECT name, 'element', namespace, COALESCE(description, '')
		FROM elements
		WHERE LOWER(name) LIKE ? OR LOWER(description) LIKE ?
		LIMIT 20
	`, likeQuery, likeQuery)
	if rows != nil {
		for rows.Next() {
			var r SearchResult
			rows.Scan(&r.Name, &r.Type, &r.Category, &r.Synopsis)
			r.ID = "element:" + r.Name
			results = append(results, r)
		}
		rows.Close()
	}

	// Search snippets
	rows, _ = a.db.Query(`
		SELECT name, 'snippet', COALESCE(scope, ''), COALESCE(description, '')
		FROM snippets
		WHERE LOWER(name) LIKE ? OR LOWER(description) LIKE ?
		LIMIT 10
	`, likeQuery, likeQuery)
	if rows != nil {
		for rows.Next() {
			var r SearchResult
			rows.Scan(&r.Name, &r.Type, &r.Category, &r.Synopsis)
			r.ID = "snippet:" + r.Name
			results = append(results, r)
		}
		rows.Close()
	}

	// Search errors
	rows, _ = a.db.Query(`
		SELECT code, 'wix-error', severity, COALESCE(description, '')
		FROM errors
		WHERE LOWER(code) LIKE ? OR LOWER(description) LIKE ?
		LIMIT 10
	`, likeQuery, likeQuery)
	if rows != nil {
		for rows.Next() {
			var r SearchResult
			rows.Scan(&r.Name, &r.Type, &r.Category, &r.Synopsis)
			r.ID = "error:" + r.Name
			results = append(results, r)
		}
		rows.Close()
	}

	// Search rules
	rows, _ = a.db.Query(`
		SELECT rule_id, 'rule', COALESCE(category, ''), COALESCE(description, '')
		FROM rules
		WHERE LOWER(rule_id) LIKE ? OR LOWER(name) LIKE ? OR LOWER(description) LIKE ?
		LIMIT 10
	`, likeQuery, likeQuery, likeQuery)
	if rows != nil {
		for rows.Next() {
			var r SearchResult
			rows.Scan(&r.Name, &r.Type, &r.Category, &r.Synopsis)
			r.ID = "rule:" + r.Name
			results = append(results, r)
		}
		rows.Close()
	}

	return results
}

// GetStats returns database statistics
func (a *App) GetStats() map[string]int {
	stats := make(map[string]int)

	var elements, attrs, snippets, errors, ice, rules int
	a.db.QueryRow(`SELECT COUNT(*) FROM elements`).Scan(&elements)
	a.db.QueryRow(`SELECT COUNT(*) FROM attributes`).Scan(&attrs)
	a.db.QueryRow(`SELECT COUNT(*) FROM snippets`).Scan(&snippets)
	a.db.QueryRow(`SELECT COUNT(*) FROM errors`).Scan(&errors)
	a.db.QueryRow(`SELECT COUNT(*) FROM ice_rules`).Scan(&ice)
	a.db.QueryRow(`SELECT COUNT(*) FROM rules`).Scan(&rules)

	stats["elements"] = elements
	stats["attributes"] = attrs
	stats["snippets"] = snippets
	stats["errors"] = errors
	stats["ice_rules"] = ice
	stats["rules"] = rules

	return stats
}
