export namespace main {
	
	export class AttributeInfo {
	    name: string;
	    type: string;
	    required: boolean;
	    default_value: string;
	    description: string;
	    enum_values: string[];
	
	    static createFrom(source: any = {}) {
	        return new AttributeInfo(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.name = source["name"];
	        this.type = source["type"];
	        this.required = source["required"];
	        this.default_value = source["default_value"];
	        this.description = source["description"];
	        this.enum_values = source["enum_values"];
	    }
	}
	export class ElementInfo {
	    id: number;
	    name: string;
	    namespace: string;
	    since_version: string;
	    description: string;
	    documentation: string;
	    remarks: string;
	    parents: string[];
	    children: string[];
	    attributes: AttributeInfo[];
	
	    static createFrom(source: any = {}) {
	        return new ElementInfo(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.id = source["id"];
	        this.name = source["name"];
	        this.namespace = source["namespace"];
	        this.since_version = source["since_version"];
	        this.description = source["description"];
	        this.documentation = source["documentation"];
	        this.remarks = source["remarks"];
	        this.parents = source["parents"];
	        this.children = source["children"];
	        this.attributes = this.convertValues(source["attributes"], AttributeInfo);
	    }
	
		convertValues(a: any, classs: any, asMap: boolean = false): any {
		    if (!a) {
		        return a;
		    }
		    if (a.slice && a.map) {
		        return (a as any[]).map(elem => this.convertValues(elem, classs));
		    } else if ("object" === typeof a) {
		        if (asMap) {
		            for (const key of Object.keys(a)) {
		                a[key] = new classs(a[key]);
		            }
		            return a;
		        }
		        return new classs(a);
		    }
		    return a;
		}
	}
	export class ErrorInfo {
	    id: number;
	    code: string;
	    severity: string;
	    message: string;
	    description: string;
	    resolution: string;
	
	    static createFrom(source: any = {}) {
	        return new ErrorInfo(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.id = source["id"];
	        this.code = source["code"];
	        this.severity = source["severity"];
	        this.message = source["message"];
	        this.description = source["description"];
	        this.resolution = source["resolution"];
	    }
	}
	export class IceRuleInfo {
	    id: number;
	    code: string;
	    severity: string;
	    description: string;
	    tables: string;
	    resolution: string;
	
	    static createFrom(source: any = {}) {
	        return new IceRuleInfo(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.id = source["id"];
	        this.code = source["code"];
	        this.severity = source["severity"];
	        this.description = source["description"];
	        this.tables = source["tables"];
	        this.resolution = source["resolution"];
	    }
	}
	export class RuleInfo {
	    id: number;
	    rule_id: string;
	    category: string;
	    severity: string;
	    name: string;
	    description: string;
	    rationale: string;
	    fix_suggestion: string;
	    target_name: string;
	
	    static createFrom(source: any = {}) {
	        return new RuleInfo(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.id = source["id"];
	        this.rule_id = source["rule_id"];
	        this.category = source["category"];
	        this.severity = source["severity"];
	        this.name = source["name"];
	        this.description = source["description"];
	        this.rationale = source["rationale"];
	        this.fix_suggestion = source["fix_suggestion"];
	        this.target_name = source["target_name"];
	    }
	}
	export class SearchResult {
	    id: string;
	    name: string;
	    type: string;
	    category: string;
	    synopsis: string;
	
	    static createFrom(source: any = {}) {
	        return new SearchResult(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.id = source["id"];
	        this.name = source["name"];
	        this.type = source["type"];
	        this.category = source["category"];
	        this.synopsis = source["synopsis"];
	    }
	}
	export class SnippetInfo {
	    id: number;
	    name: string;
	    prefix: string;
	    description: string;
	    body: string;
	    scope: string;
	
	    static createFrom(source: any = {}) {
	        return new SnippetInfo(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.id = source["id"];
	        this.name = source["name"];
	        this.prefix = source["prefix"];
	        this.description = source["description"];
	        this.body = source["body"];
	        this.scope = source["scope"];
	    }
	}
	export class TreeNode {
	    id: string;
	    name: string;
	    type: string;
	    icon: string;
	    children?: TreeNode[];
	
	    static createFrom(source: any = {}) {
	        return new TreeNode(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.id = source["id"];
	        this.name = source["name"];
	        this.type = source["type"];
	        this.icon = source["icon"];
	        this.children = this.convertValues(source["children"], TreeNode);
	    }
	
		convertValues(a: any, classs: any, asMap: boolean = false): any {
		    if (!a) {
		        return a;
		    }
		    if (a.slice && a.map) {
		        return (a as any[]).map(elem => this.convertValues(elem, classs));
		    } else if ("object" === typeof a) {
		        if (asMap) {
		            for (const key of Object.keys(a)) {
		                a[key] = new classs(a[key]);
		            }
		            return a;
		        }
		        return new classs(a);
		    }
		    return a;
		}
	}

}

