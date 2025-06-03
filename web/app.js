import { invoke } from '@tauri-apps/api/core';

class DhdApp {
	constructor() {
		this.modules = [];
		this.selectedModules = new Set();
		this.tags = new Set();
		this.activeTags = new Set();
		
		this.init();
	}
	
	async init() {
		await this.loadModules();
		this.setupEventListeners();
	}
	
	async loadModules() {
		try {
			this.modules = await invoke('list_modules');
			this.extractTags();
			this.renderTags();
			this.renderModules();
		} catch (error) {
			console.error('Failed to load modules:', error);
		}
	}
	
	extractTags() {
		this.tags.clear();
		this.modules.forEach(module => {
			module.tags.forEach(tag => this.tags.add(tag));
		});
	}
	
	setupEventListeners() {
		document.getElementById('search').addEventListener('input', (e) => {
			this.filterModules(e.target.value);
		});
		
		document.getElementById('select-all').addEventListener('click', () => {
			this.selectAllVisible();
		});
		
		document.getElementById('deselect-all').addEventListener('click', () => {
			this.deselectAll();
		});
		
		document.getElementById('plan-btn').addEventListener('click', () => {
			this.generatePlan();
		});
		
		document.getElementById('apply-btn').addEventListener('click', () => {
			this.applyModules();
		});
	}
	
	renderTags() {
		const container = document.getElementById('tags-filter');
		container.innerHTML = '';
		
		this.tags.forEach(tag => {
			const tagEl = document.createElement('span');
			tagEl.className = 'tag';
			tagEl.textContent = tag;
			tagEl.addEventListener('click', () => this.toggleTag(tag, tagEl));
			container.appendChild(tagEl);
		});
	}
	
	toggleTag(tag, element) {
		if (this.activeTags.has(tag)) {
			this.activeTags.delete(tag);
			element.classList.remove('active');
		} else {
			this.activeTags.add(tag);
			element.classList.add('active');
		}
		this.filterModules(document.getElementById('search').value);
	}
	
	filterModules(searchTerm = '') {
		const filtered = this.modules.filter(module => {
			const matchesSearch = module.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
				(module.description && module.description.toLowerCase().includes(searchTerm.toLowerCase()));
			
			const matchesTags = this.activeTags.size === 0 || 
				module.tags.some(tag => this.activeTags.has(tag));
			
			return matchesSearch && matchesTags;
		});
		
		this.renderModules(filtered);
	}
	
	renderModules(modules = this.modules) {
		const container = document.getElementById('modules-list');
		container.innerHTML = '';
		
		modules.forEach(module => {
			const moduleEl = document.createElement('div');
			moduleEl.className = 'module-item';
			if (this.selectedModules.has(module.name)) {
				moduleEl.classList.add('selected');
			}
			
			moduleEl.innerHTML = `
				<h4>${module.name}</h4>
				${module.description ? `<p>${module.description}</p>` : ''}
				${module.dependencies.length > 0 ? 
					`<div class="module-deps">Depends on: ${module.dependencies.join(', ')}</div>` : ''}
			`;
			
			moduleEl.addEventListener('click', () => this.toggleModule(module, moduleEl));
			container.appendChild(moduleEl);
		});
	}
	
	toggleModule(module, element) {
		if (this.selectedModules.has(module.name)) {
			this.selectedModules.delete(module.name);
			element.classList.remove('selected');
		} else {
			this.selectedModules.add(module.name);
			element.classList.add('selected');
		}
		
		this.updateButtons();
	}
	
	selectAllVisible() {
		const visibleModules = document.querySelectorAll('.module-item');
		visibleModules.forEach((el, index) => {
			const moduleName = this.modules[index].name;
			this.selectedModules.add(moduleName);
			el.classList.add('selected');
		});
		this.updateButtons();
	}
	
	deselectAll() {
		this.selectedModules.clear();
		document.querySelectorAll('.module-item').forEach(el => {
			el.classList.remove('selected');
		});
		this.updateButtons();
	}
	
	updateButtons() {
		const hasSelection = this.selectedModules.size > 0;
		document.getElementById('plan-btn').disabled = !hasSelection;
	}
	
	async generatePlan() {
		try {
			const modules = Array.from(this.selectedModules);
			const plan = await invoke('generate_plan', { modules });
			this.displayPlan(plan);
		} catch (error) {
			console.error('Failed to generate plan:', error);
		}
	}
	
	displayPlan(plan) {
		const planView = document.getElementById('plan-view');
		const planContent = document.getElementById('plan-content');
		
		planContent.innerHTML = '';
		
		// Add summary
		const summary = document.createElement('div');
		summary.className = 'plan-summary';
		summary.innerHTML = `<h3>Execution Plan</h3><p>Total actions: ${plan.total_actions}</p>`;
		planContent.appendChild(summary);
		
		// Display each module and its actions
		plan.modules.forEach(module => {
			const moduleEl = document.createElement('div');
			moduleEl.className = 'module-plan';
			
			let html = `<h4>${module.name}`;
			if (module.dependencies.length > 0) {
				html += ` <span class="deps">(depends on: ${module.dependencies.join(', ')})</span>`;
			}
			html += '</h4>';
			
			if (module.description) {
				html += `<p class="module-desc">${module.description}</p>`;
			}
			
			moduleEl.innerHTML = html;
			
			// Add actions for this module
			if (module.actions.length > 0) {
				const actionsList = document.createElement('div');
				actionsList.className = 'actions-list';
				
				module.actions.forEach((action, index) => {
					const actionEl = document.createElement('div');
					actionEl.className = 'action-item';
					actionEl.innerHTML = `${index + 1}. ${action.description}`;
					
					// Add atoms details if available
					if (action.atoms && action.atoms.length > 0) {
						const atomsList = document.createElement('div');
						atomsList.className = 'atoms-list';
						action.atoms.forEach(atom => {
							const atomEl = document.createElement('div');
							atomEl.className = 'atom-item';
							atomEl.innerHTML = `→ ${atom}`;
							atomsList.appendChild(atomEl);
						});
						actionEl.appendChild(atomsList);
					}
					
					actionsList.appendChild(actionEl);
				});
				
				moduleEl.appendChild(actionsList);
			}
			
			planContent.appendChild(moduleEl);
		});
		
		planView.classList.remove('hidden');
		document.getElementById('apply-btn').disabled = false;
	}
	
	async applyModules() {
		const progressView = document.getElementById('progress-view');
		const progressFill = document.getElementById('progress-fill');
		const progressLog = document.getElementById('progress-log');
		
		progressView.classList.remove('hidden');
		progressLog.innerHTML = '';
		
		try {
			const modules = Array.from(this.selectedModules);
			const result = await invoke('apply_modules', { modules });
			
			// Simulate progress
			let progress = 0;
			const interval = setInterval(() => {
				progress += 10;
				progressFill.style.width = `${progress}%`;
				
				if (progress >= 100) {
					clearInterval(interval);
					progressLog.innerHTML += `\n✅ ${result}`;
				}
			}, 200);
			
		} catch (error) {
			console.error('Failed to apply modules:', error);
			progressLog.innerHTML += `\n❌ Error: ${error}`;
		}
	}
}

// Initialize app when DOM is ready
document.addEventListener('DOMContentLoaded', () => {
	new DhdApp();
});