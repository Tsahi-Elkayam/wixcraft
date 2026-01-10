//! Dependency analyzer module
//!
//! Analyzes and manages dependencies in WiX installer projects.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

/// Dependency type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DependencyType {
    /// .NET Framework
    DotNetFramework,
    /// .NET Core/.NET 5+
    DotNetCore,
    /// Visual C++ Runtime
    VCRuntime,
    /// DirectX
    DirectX,
    /// Windows SDK
    WindowsSdk,
    /// Native DLL
    NativeDll,
    /// COM component
    ComComponent,
    /// WiX extension
    WixExtension,
    /// Merge module
    MergeModule,
    /// Other
    Other,
}

impl DependencyType {
    pub fn as_str(&self) -> &'static str {
        match self {
            DependencyType::DotNetFramework => ".NET Framework",
            DependencyType::DotNetCore => ".NET",
            DependencyType::VCRuntime => "Visual C++ Runtime",
            DependencyType::DirectX => "DirectX",
            DependencyType::WindowsSdk => "Windows SDK",
            DependencyType::NativeDll => "Native DLL",
            DependencyType::ComComponent => "COM Component",
            DependencyType::WixExtension => "WiX Extension",
            DependencyType::MergeModule => "Merge Module",
            DependencyType::Other => "Other",
        }
    }
}

/// Dependency information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub name: String,
    pub version: Option<String>,
    pub dep_type: DependencyType,
    pub required: bool,
    pub bundled: bool,
    pub source_file: Option<PathBuf>,
    pub download_url: Option<String>,
}

impl Dependency {
    pub fn new(name: &str, dep_type: DependencyType) -> Self {
        Self {
            name: name.to_string(),
            version: None,
            dep_type,
            required: true,
            bundled: false,
            source_file: None,
            download_url: None,
        }
    }

    pub fn with_version(mut self, version: &str) -> Self {
        self.version = Some(version.to_string());
        self
    }

    pub fn optional(mut self) -> Self {
        self.required = false;
        self
    }

    pub fn bundled(mut self) -> Self {
        self.bundled = true;
        self
    }

    pub fn with_source(mut self, path: PathBuf) -> Self {
        self.source_file = Some(path);
        self
    }

    pub fn with_download_url(mut self, url: &str) -> Self {
        self.download_url = Some(url.to_string());
        self
    }
}

/// Dependency graph node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyNode {
    pub dependency: Dependency,
    pub depends_on: Vec<String>,
    pub required_by: Vec<String>,
}

impl DependencyNode {
    pub fn new(dependency: Dependency) -> Self {
        Self {
            dependency,
            depends_on: Vec::new(),
            required_by: Vec::new(),
        }
    }

    pub fn add_dependency(&mut self, name: &str) {
        if !self.depends_on.contains(&name.to_string()) {
            self.depends_on.push(name.to_string());
        }
    }

    pub fn add_required_by(&mut self, name: &str) {
        if !self.required_by.contains(&name.to_string()) {
            self.required_by.push(name.to_string());
        }
    }
}

/// Dependency graph
#[derive(Debug, Clone, Default)]
pub struct DependencyGraph {
    nodes: HashMap<String, DependencyNode>,
}

impl DependencyGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_dependency(&mut self, dep: Dependency) {
        let name = dep.name.clone();
        self.nodes.insert(name, DependencyNode::new(dep));
    }

    pub fn add_edge(&mut self, from: &str, to: &str) {
        if let Some(node) = self.nodes.get_mut(from) {
            node.add_dependency(to);
        }
        if let Some(node) = self.nodes.get_mut(to) {
            node.add_required_by(from);
        }
    }

    pub fn get(&self, name: &str) -> Option<&DependencyNode> {
        self.nodes.get(name)
    }

    pub fn get_all(&self) -> impl Iterator<Item = &DependencyNode> {
        self.nodes.values()
    }

    pub fn get_root_dependencies(&self) -> Vec<&DependencyNode> {
        self.nodes
            .values()
            .filter(|n| n.required_by.is_empty())
            .collect()
    }

    pub fn get_leaf_dependencies(&self) -> Vec<&DependencyNode> {
        self.nodes
            .values()
            .filter(|n| n.depends_on.is_empty())
            .collect()
    }

    /// Detect circular dependencies
    pub fn detect_cycles(&self) -> Vec<Vec<String>> {
        let mut cycles = Vec::new();
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut path = Vec::new();

        for name in self.nodes.keys() {
            if !visited.contains(name) {
                self.detect_cycles_dfs(name, &mut visited, &mut rec_stack, &mut path, &mut cycles);
            }
        }

        cycles
    }

    fn detect_cycles_dfs(
        &self,
        name: &str,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
        path: &mut Vec<String>,
        cycles: &mut Vec<Vec<String>>,
    ) {
        visited.insert(name.to_string());
        rec_stack.insert(name.to_string());
        path.push(name.to_string());

        if let Some(node) = self.nodes.get(name) {
            for dep in &node.depends_on {
                if !visited.contains(dep) {
                    self.detect_cycles_dfs(dep, visited, rec_stack, path, cycles);
                } else if rec_stack.contains(dep) {
                    // Found a cycle
                    let cycle_start = path.iter().position(|n| n == dep).unwrap();
                    let cycle: Vec<_> = path[cycle_start..].to_vec();
                    cycles.push(cycle);
                }
            }
        }

        path.pop();
        rec_stack.remove(name);
    }

    /// Get topological order
    pub fn topological_sort(&self) -> Result<Vec<String>, String> {
        let cycles = self.detect_cycles();
        if !cycles.is_empty() {
            return Err(format!(
                "Circular dependency detected: {}",
                cycles[0].join(" -> ")
            ));
        }

        let mut result = Vec::new();
        let mut visited = HashSet::new();

        for name in self.nodes.keys() {
            if !visited.contains(name) {
                self.topo_dfs(name, &mut visited, &mut result);
            }
        }

        result.reverse();
        Ok(result)
    }

    fn topo_dfs(&self, name: &str, visited: &mut HashSet<String>, result: &mut Vec<String>) {
        visited.insert(name.to_string());

        if let Some(node) = self.nodes.get(name) {
            for dep in &node.depends_on {
                if !visited.contains(dep) {
                    self.topo_dfs(dep, visited, result);
                }
            }
        }

        result.push(name.to_string());
    }
}

/// Dependency analyzer
pub struct DependencyAnalyzer;

impl DependencyAnalyzer {
    /// Analyze a binary file for dependencies
    pub fn analyze_binary(_path: &PathBuf) -> Vec<Dependency> {
        // Simulated analysis - in production would parse PE headers
        Vec::new()
    }

    /// Analyze a WiX source file for dependencies
    pub fn analyze_wix_source(_path: &PathBuf) -> Vec<Dependency> {
        // Would parse WiX XML for extension references, merge modules, etc.
        Vec::new()
    }

    /// Check if VC++ runtime is needed
    pub fn needs_vcruntime(_binaries: &[PathBuf]) -> Option<String> {
        // Would check for MSVCR*.dll or VCRUNTIME*.dll dependencies
        None
    }

    /// Check .NET version required
    pub fn get_dotnet_requirement(_binaries: &[PathBuf]) -> Option<(DependencyType, String)> {
        // Would analyze .NET assemblies for target framework
        None
    }
}

/// Dependency report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyReport {
    pub project_name: String,
    pub total_dependencies: usize,
    pub bundled_dependencies: usize,
    pub external_dependencies: usize,
    pub by_type: HashMap<String, usize>,
    pub missing_dependencies: Vec<String>,
    pub suggestions: Vec<String>,
}

impl DependencyReport {
    pub fn generate(project_name: &str, graph: &DependencyGraph) -> Self {
        let mut by_type: HashMap<String, usize> = HashMap::new();
        let mut bundled = 0;
        let mut external = 0;

        for node in graph.get_all() {
            let type_str = node.dependency.dep_type.as_str().to_string();
            *by_type.entry(type_str).or_insert(0) += 1;

            if node.dependency.bundled {
                bundled += 1;
            } else {
                external += 1;
            }
        }

        Self {
            project_name: project_name.to_string(),
            total_dependencies: graph.nodes.len(),
            bundled_dependencies: bundled,
            external_dependencies: external,
            by_type,
            missing_dependencies: Vec::new(),
            suggestions: Vec::new(),
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }
}

/// WiX extension dependency helper
pub struct WixExtensionHelper;

impl WixExtensionHelper {
    /// Get common WiX extensions and their uses
    pub fn get_extension_info(name: &str) -> Option<(&'static str, &'static str)> {
        match name {
            "WixUIExtension" => Some(("UI Dialogs", "Provides standard UI dialog sets")),
            "WixUtilExtension" => Some(("Utilities", "User creation, file search, etc.")),
            "WixNetFxExtension" => Some((".NET", ".NET Framework detection")),
            "WixFirewallExtension" => Some(("Firewall", "Windows Firewall rules")),
            "WixIIsExtension" => Some(("IIS", "IIS website and app pool configuration")),
            "WixSqlExtension" => Some(("SQL", "SQL Server database operations")),
            _ => None,
        }
    }

    /// Check if extension is referenced in WiX source
    pub fn is_extension_used(_source: &str, extension: &str) -> bool {
        // Would parse XML and check for extension usage
        extension.len() > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dependency_type_as_str() {
        assert_eq!(DependencyType::DotNetFramework.as_str(), ".NET Framework");
        assert_eq!(DependencyType::VCRuntime.as_str(), "Visual C++ Runtime");
    }

    #[test]
    fn test_dependency_new() {
        let dep = Dependency::new("MyLib", DependencyType::NativeDll);
        assert_eq!(dep.name, "MyLib");
        assert!(dep.required);
    }

    #[test]
    fn test_dependency_with_version() {
        let dep = Dependency::new("MyLib", DependencyType::NativeDll).with_version("1.0.0");
        assert_eq!(dep.version, Some("1.0.0".to_string()));
    }

    #[test]
    fn test_dependency_optional() {
        let dep = Dependency::new("MyLib", DependencyType::NativeDll).optional();
        assert!(!dep.required);
    }

    #[test]
    fn test_dependency_bundled() {
        let dep = Dependency::new("MyLib", DependencyType::NativeDll).bundled();
        assert!(dep.bundled);
    }

    #[test]
    fn test_dependency_node_new() {
        let dep = Dependency::new("MyLib", DependencyType::NativeDll);
        let node = DependencyNode::new(dep);
        assert!(node.depends_on.is_empty());
    }

    #[test]
    fn test_dependency_node_add_dependency() {
        let dep = Dependency::new("MyLib", DependencyType::NativeDll);
        let mut node = DependencyNode::new(dep);
        node.add_dependency("OtherLib");
        assert_eq!(node.depends_on.len(), 1);
    }

    #[test]
    fn test_dependency_graph_new() {
        let graph = DependencyGraph::new();
        assert!(graph.nodes.is_empty());
    }

    #[test]
    fn test_dependency_graph_add() {
        let mut graph = DependencyGraph::new();
        let dep = Dependency::new("MyLib", DependencyType::NativeDll);
        graph.add_dependency(dep);
        assert!(graph.get("MyLib").is_some());
    }

    #[test]
    fn test_dependency_graph_add_edge() {
        let mut graph = DependencyGraph::new();
        graph.add_dependency(Dependency::new("A", DependencyType::NativeDll));
        graph.add_dependency(Dependency::new("B", DependencyType::NativeDll));
        graph.add_edge("A", "B");

        let node_a = graph.get("A").unwrap();
        assert!(node_a.depends_on.contains(&"B".to_string()));
    }

    #[test]
    fn test_dependency_graph_roots() {
        let mut graph = DependencyGraph::new();
        graph.add_dependency(Dependency::new("A", DependencyType::NativeDll));
        graph.add_dependency(Dependency::new("B", DependencyType::NativeDll));
        graph.add_edge("A", "B");

        let roots = graph.get_root_dependencies();
        assert_eq!(roots.len(), 1);
        assert_eq!(roots[0].dependency.name, "A");
    }

    #[test]
    fn test_dependency_graph_leaves() {
        let mut graph = DependencyGraph::new();
        graph.add_dependency(Dependency::new("A", DependencyType::NativeDll));
        graph.add_dependency(Dependency::new("B", DependencyType::NativeDll));
        graph.add_edge("A", "B");

        let leaves = graph.get_leaf_dependencies();
        assert_eq!(leaves.len(), 1);
        assert_eq!(leaves[0].dependency.name, "B");
    }

    #[test]
    fn test_dependency_graph_no_cycles() {
        let mut graph = DependencyGraph::new();
        graph.add_dependency(Dependency::new("A", DependencyType::NativeDll));
        graph.add_dependency(Dependency::new("B", DependencyType::NativeDll));
        graph.add_edge("A", "B");

        let cycles = graph.detect_cycles();
        assert!(cycles.is_empty());
    }

    #[test]
    fn test_dependency_graph_topological_sort() {
        let mut graph = DependencyGraph::new();
        graph.add_dependency(Dependency::new("A", DependencyType::NativeDll));
        graph.add_dependency(Dependency::new("B", DependencyType::NativeDll));
        graph.add_dependency(Dependency::new("C", DependencyType::NativeDll));
        graph.add_edge("A", "B");
        graph.add_edge("B", "C");

        let order = graph.topological_sort().unwrap();
        assert_eq!(order, vec!["A", "B", "C"]);
    }

    #[test]
    fn test_dependency_report_generate() {
        let mut graph = DependencyGraph::new();
        graph.add_dependency(Dependency::new("A", DependencyType::NativeDll).bundled());
        graph.add_dependency(Dependency::new("B", DependencyType::VCRuntime));

        let report = DependencyReport::generate("MyProject", &graph);
        assert_eq!(report.total_dependencies, 2);
        assert_eq!(report.bundled_dependencies, 1);
        assert_eq!(report.external_dependencies, 1);
    }

    #[test]
    fn test_dependency_report_to_json() {
        let graph = DependencyGraph::new();
        let report = DependencyReport::generate("MyProject", &graph);
        let json = report.to_json();
        assert!(json.contains("MyProject"));
    }

    #[test]
    fn test_wix_extension_helper_get_info() {
        let info = WixExtensionHelper::get_extension_info("WixUIExtension");
        assert!(info.is_some());
        assert_eq!(info.unwrap().0, "UI Dialogs");
    }

    #[test]
    fn test_wix_extension_helper_unknown() {
        let info = WixExtensionHelper::get_extension_info("UnknownExtension");
        assert!(info.is_none());
    }
}
